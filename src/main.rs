#![deny(warnings)]
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use anyhow::Result;
use aws_types::SdkConfig;
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use warp::Filter;

mod my_aws;
mod types;

use my_aws::{MyAwsConfig, MyAwsLambda};
use types::{MyRabbitEvent, QueueDefinition};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = MyAwsConfig::new("env").await;
    info!("aws_config={:?}", config);

    let rabbitmq_uri = match env::var("RABBITMQ_URI") {
        Ok(uri) => uri,
        Err(_) => "amqp://guest:guest@127.0.0.1".to_string(),
    };
    info!("rabbitmq_uri={:?}", rabbitmq_uri);

    // async block으로 소유권을 넘긴다
    let config_http = config.clone();
    let config_amqp = config.clone();

    let _handle_http = tokio::spawn(async move { main_http(&config_http).await });

    // amqp를 tokio::spawn으로 실행하려고 하면
    // error: future cannot be sent between threads safely
    // 가 발생하는게 어떻게 고칠지 몰라서 우회. 직접 띄우면 상관없더라
    let _reuslt_amqp = main_amqp(&config_amqp, &rabbitmq_uri).await;

    let _x = QueueDefinition {
        queue: "a".to_string(),
        region: "a".to_string(),
        function_name: "a".to_string(),
    };
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

#[allow(unused)]
async fn main_amqp(config: &SdkConfig, rabbitmq_uri: &str) -> Result<()> {
    // TODO: aws client 재사용하면 http 소켓 연결을 재사용할 수 있을거같은데
    let client = aws_sdk_lambda::Client::new(config);
    let lambda = MyAwsLambda::new(client);

    // TODO: 큐 이름을 나눌 필요성?
    let queue = "hello";
    let function_name = "mikoto-sample-dev-commonDequeue";

    let mut connection = if String::from(rabbitmq_uri).starts_with("amqp://") {
        Connection::insecure_open(rabbitmq_uri)?
    } else {
        Connection::open(rabbitmq_uri)?
    };

    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(queue, QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let event = MyRabbitEvent::new(delivery.clone());
                let event_bytes = event.to_string().into_bytes();

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
