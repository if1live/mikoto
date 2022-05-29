#![deny(warnings)]
use anyhow::Result;
use aws_types::SdkConfig;
use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use warp::Filter;

mod my_aws;
mod types;

use my_aws::{MyAwsConfig, MyAwsLambda};
use types::{MikotoDefinition, MyRabbitEvent, QueueDefinition};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    let config = MyAwsConfig::new("offline").await;
    info!("aws_config={:?}", config);

    let definition = MikotoDefinition {
        queues: vec![QueueDefinition {
            queue: "a".to_string(),
            function_name: "a".to_string(),
        }],
    };

    let rabbitmq_uri = match env::var("RABBITMQ_URI") {
        Ok(uri) => uri,
        Err(_) => "amqp://guest:guest@127.0.0.1".to_string(),
    };
    info!("rabbitmq_uri={:?}", rabbitmq_uri);

    let options = ConnectionProperties::default();

    let connection = Connection::connect(&rabbitmq_uri, options).await.unwrap();

    // async block으로 소유권을 넘긴다
    let config_http = config.clone();
    let config_amqp = config.clone();

    let _handle_http = tokio::spawn(async move { main_http(&config_http).await });
    let _handle_http =
        tokio::spawn(async move { main_amqp(config_amqp, connection, &definition).await });

    // TODO: 루프 안멈추는 더 멀쩡한 방법?
    sleep(Duration::from_millis(100000)).await;
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
async fn main_amqp(
    config: SdkConfig,
    mut connection: Connection,
    definition: &MikotoDefinition,
) -> Result<()> {
    // TODO: aws client 재사용하면 http 소켓 연결을 재사용할 수 있을거같은데
    // let client = aws_sdk_lambda::Client::new(config);
    // let lambda = MyAwsLambda::new(client);

    let queue_name = "hello";
    let function_name = "mikoto-sample-dev-commonDequeue";

    /*
    let config0 = config.clone();
    let channel = connection.open_channel(None).unwrap();
    let queue = channel
        .queue_declare("TODO", QueueDeclareOptions::default())
        .unwrap();
    let consumer = queue.consume(ConsumerOptions::default()).unwrap();

    let _handle = tokio::spawn(async move {
        let queue_name = "hello";
        let function_name = "mikoto-sample-dev-commonDequeue";

        for (i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let event = MyRabbitEvent::new(delivery.clone());
                    let event_bytes = event.to_string().into_bytes();

                    let client = aws_sdk_lambda::Client::new(&config0);
                    let lambda = MyAwsLambda::new(client);
                    // let _result = lambda.invoke(function_name, &event_bytes).await;

                    consumer.ack(delivery).unwrap();
                }
                other => {
                    println!("Consumer ended: ${:?}", other);
                    break;
                }
            }
        }
        true
    });
    _handle.await;

    // connection.close()?;
    Ok(())
    */

    let channel = connection.create_channel().await.unwrap();

    let _queue = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let consumer = channel
        .basic_consume(
            queue_name,
            "tag_foo",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = match delivery {
            // Carries the delivery alongside its channel
            Ok(Some(delivery)) => {
                println!("{:?}", delivery);
                delivery
            }
            // The consumer got canceled
            Ok(None) => return,
            // Carries the error and is always followed by Ok(None)
            Err(error) => {
                dbg!("Failed to consume queue message {}", error);
                return;
            }
        };

        // Do something with the delivery data (The message payload)
        let event_text = MyRabbitEvent::to_string(&delivery);
        println!("{}", event_text);

        let event_bytes = event_text.into_bytes();

        delivery
            .ack(BasicAckOptions::default())
            .await
            .expect("Failed to ack send_webhook_event message");

        // TODO: 더 멀쩡한 방법?
        let config = MyAwsConfig::new("offline").await;
        let client = aws_sdk_lambda::Client::new(&config);
        let lambda = MyAwsLambda::new(client);
        let _result = lambda.invoke(function_name, &event_bytes).await;
    });

    Ok(())
}
