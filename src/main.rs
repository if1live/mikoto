#![deny(warnings)]
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
};
use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_lambda::Region;
use aws_smithy_http::endpoint::Endpoint;
use aws_types::{credentials::SharedCredentialsProvider, Credentials, SdkConfig};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::hyper::Uri;
use warp::Filter;

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    s_val: String,
    i_val: i32,
    f_val: f32,
}

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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[allow(unused)]
async fn main_amqp(config: &SdkConfig) -> Result<()> {
    let queue = "hello";

    // TODO: secure open?
    let mut connection = Connection::insecure_open(RABBITMQ_URI)?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(queue, QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let function_name = "mikoto-sample-dev-commonDequeue";
                let body0 = delivery.body.clone();
                // println!("aaa [{}] {}", i, String::from_utf8_lossy(&delivery.body));
                // TODO: 이벤트 규격이 람다 규격과 비슷하도록 만들기

                // TODO: aws client 재사용하면 http 소켓 연결을 재사용할 수 있을거같은데
                let config0 = config.clone();
                let client = aws_sdk_lambda::Client::new(&config0);
                let lambda = MyAwsLambda::new(client);

                let payload = body0.as_slice();
                lambda.invoke(function_name, payload).await;

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

#[allow(dead_code)]
#[derive(Clone)]
struct MyAwsLambda {
    client: aws_sdk_lambda::Client,
}
impl MyAwsLambda {
    #[allow(dead_code)]
    pub fn new(client: aws_sdk_lambda::Client) -> Self {
        MyAwsLambda { client: client }
    }

    #[allow(dead_code)]
    pub async fn invoke(&self, function_name: &str, payload: &[u8]) -> Result<()> {
        let blob = aws_smithy_types::Blob::new(payload);
        let response = self
            .client
            .invoke()
            .function_name(function_name)
            .payload(blob)
            .send()
            .await?;

        println!("Response from invoke: {:#?}", response);

        Ok(())
    }
}

#[allow(dead_code)]
fn main_consume() -> amiquip::Result<()> {
    let mut connection = Connection::insecure_open(RABBITMQ_URI)?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare("hello", QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages. Press Ctrl-C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                println!("({:>3} Received [{}])", i, body);
                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: ${:?}", other);
                break;
            }
        }
    }

    connection.close()
}

#[allow(dead_code)]
fn main_produce() -> amiquip::Result<()> {
    let mut connection = Connection::insecure_open(RABBITMQ_URI)?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let body = format!("{:?}", since_the_epoch);

    exchange.publish(Publish::new(body.as_bytes(), "hello"))?;

    connection.close()
}
