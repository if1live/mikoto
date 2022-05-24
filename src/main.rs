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
use serde_json::json;
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

    // main_http().await
    main_lambda(&_config_offline).await
    // let _x = main_consume();
    // let _x = main_produce();
    // return;
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

#[allow(dead_code)]
async fn main_http() {
    // Match any request and return hello world!
    let routes = warp::any().map(|| "Hello, World!");

    warp::serve(routes)
        // ipv6 + ipv6 any addr
        .run(([0, 0, 0, 0, 0, 0, 0, 0], 8080))
        .await;
}

#[allow(dead_code)]
async fn main_lambda(config: &SdkConfig) {
    let client = aws_sdk_lambda::Client::new(config);
    let lambda = MyAwsLambda::new(client);
    let result = lambda.invoke().await;

    println!("{:?}", &result);
    return;
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
struct MyAwsLambda {
    client: aws_sdk_lambda::Client,
}
impl MyAwsLambda {
    #[allow(dead_code)]
    pub fn new(client: aws_sdk_lambda::Client) -> Self {
        MyAwsLambda { client: client }
    }

    #[allow(dead_code)]
    pub async fn invoke(&self) -> Result<()> {
        let function_name = "mikoto-sample-dev-commonDequeue";

        // TODO:
        let command = json!({
            "s_val": "Hello".to_string(),
            "i_val": 42,
            "f_val": 3.14,
        });

        let text = serde_json::to_string(&command)?;
        let blob = aws_smithy_types::Blob::new(text.as_bytes());
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
