#![deny(warnings)]
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::event::rabbitmq::{RabbitMqBasicProperties, RabbitMqEvent, RabbitMqMessage};
use aws_sdk_lambda::Region;
use aws_smithy_http::endpoint::Endpoint;
use aws_types::{credentials::SharedCredentialsProvider, Credentials, SdkConfig};
use serde_json::json;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::hyper::Uri;
use warp::Filter;

// TODO: 환경변수로 교체하기
const RABBITMQ_URI: &'static str = "amqp://guest:guest@127.0.0.1";

#[tokio::main]
async fn main() {
    // TODO: 디버깅용으로 양쪽 설정을 남겨둠
    let _config_region = MyAwsConfig::from_region().await;
    let _config_offline = MyAwsConfig::from_offline().await;

    let config = _config_offline;

    // async block으로 소유권을 넘긴다
    let config_http = config.clone();
    let config_amqp = config.clone();

    let _handle_http = tokio::spawn(async move { main_http(&config_http).await });

    // amqp를 tokio::spawn으로 실행하려고 하면
    // error: future cannot be sent between threads safely
    // 가 발생하는게 어떻게 고칠지 몰라서 우회. 직접 띄우면 상관없더라
    let _reuslt_amqp = main_amqp(&config_amqp).await;
}

async fn main_http(_config: &SdkConfig) -> Result<()> {
    // let routes = warp::any().map(|| "Hello, World!");

    let index = warp::path::end().map(|| "hello world");
    let demo_lambda = warp::path!("demo" / "lambda").map(|| "Hello, lambda!");

    let routes = warp::get().and(index.or(demo_lambda));

    // curl http://[::1]:8080/
    let addr = IpAddr::from_str("::0").unwrap();
    warp::serve(routes).run((addr, 8080)).await;

    Ok(())
}

#[allow(unused)]
async fn main_amqp(config: &SdkConfig) -> Result<()> {
    // TODO: 큐 이름을 나눌 필요성?
    let queue = "hello";

    let mut connection = if String::from(RABBITMQ_URI).starts_with("amqp://") {
        Connection::insecure_open(RABBITMQ_URI)?
    } else {
        Connection::open(RABBITMQ_URI)?
    };

    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(queue, QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let function_name = "mikoto-sample-dev-commonDequeue";
                let body_str = String::from_utf8_lossy(&delivery.body);

                // TODO: 이벤트 규격이 람다 규격과 비슷하도록 만들기
                let property = RabbitMqBasicProperties {
                    content_type: Some("text/plain".to_string()),
                    content_encoding: None,
                    // TODO: delivery에서 뜯을수 있나?
                    headers: HashMap::new(),
                    delivery_mode: 1,
                    priority: 34,
                    correlation_id: None,
                    reply_to: None,
                    // TODO: 제대로된 값으로 만들기
                    expiration: Some("60000".to_string()),
                    message_id: None,
                    // TODO: 현재시간 + 규격은 "Jan 1, 1970, 12:33:41 AM",
                    timestamp: Some("Jan 1, 1970, 12:33:41 AM".to_string()),
                    type_: None,
                    user_id: None,
                    app_id: None,
                    cluster_id: None,
                    body_size: 9999,
                };

                // TODO: data는 base64 json으로 추정
                let message = RabbitMqMessage {
                    basic_properties: property,
                    data: Some(body_str.to_string()),
                    redelivered: delivery.redelivered,
                };

                let event = RabbitMqEvent {
                    event_source: Some("aws:rmq".to_string()),
                    event_source_arn: Some("arn:aws:mq:us-east-1:123456789012:broker:mikoto:b-af0b701e-db74-11ec-9d64-0242ac120002".to_string()),
                    messages_by_queue: HashMap::from([
                        (delivery.routing_key.clone(), vec![message]),
                    ]),
                };

                let event_text = json!(event);
                let event_bytes = event_text.to_string().into_bytes();

                // TODO: aws client 재사용하면 http 소켓 연결을 재사용할 수 있을거같은데
                let config0 = config.clone();
                let client = aws_sdk_lambda::Client::new(&config0);
                let lambda = MyAwsLambda::new(client);

                let _result = lambda.invoke(function_name, &event_bytes).await;

                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: ${:?}", other);
                break;
            }
        }
    }

    connection.close()?;
    Ok(())
}

struct MyAwsConfig {}
impl MyAwsConfig {
    pub async fn from_offline() -> aws_types::SdkConfig {
        let region = Region::new("local");
        let endpoint = Endpoint::immutable(Uri::from_static("http://localhost:3002/"));
        let credential = Credentials::new(
            "localAccessKey",
            "localSecretAccessKey",
            None,
            None,
            "local",
        );

        let mut builder = SdkConfig::builder();
        builder.set_region(region);
        builder.set_endpoint_resolver(Some(Arc::new(endpoint)));
        builder.set_credentials_provider(Some(SharedCredentialsProvider::new(credential)));
        builder.build()
    }

    pub async fn from_region() -> aws_types::SdkConfig {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-northeast-1");
        aws_config::from_env().region(region_provider).load().await
    }
}

#[derive(Clone)]
struct MyAwsLambda {
    client: aws_sdk_lambda::Client,
}
impl MyAwsLambda {
    pub fn new(client: aws_sdk_lambda::Client) -> Self {
        MyAwsLambda { client: client }
    }

    pub async fn invoke(&self, function_name: &str, payload: &[u8]) -> Result<()> {
        let blob = aws_smithy_types::Blob::new(payload);
        let _response = self
            .client
            .invoke()
            .function_name(function_name)
            .payload(blob)
            .send()
            .await?;

        // println!("Response from invoke: {:#?}", response);

        Ok(())
    }
}
