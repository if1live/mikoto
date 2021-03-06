#![deny(warnings)]
use anyhow::Result;
use aws_types::SdkConfig;
use dotenv::dotenv;
use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::join;
use warp::Filter;

mod my_aws;
mod types;

use my_aws::{MyAwsConfig, MyAwsLambda};
use types::{MyRabbitEvent, Settings};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let config = MyAwsConfig::new();
    let config = match config {
        Ok(config) => config,
        Err(err) => panic!("{:?}", err),
    };
    info!("aws_config={:?}", config);

    let settings = Settings::new();
    let settings = match settings {
        Ok(settings) => settings,
        Err(err) => panic!("{:?}", err),
    };
    info!("settings={:?}", settings);

    let rabbitmq_uri = match env::var("RABBITMQ_URI") {
        Ok(uri) => uri,
        Err(_) => "amqp://guest:guest@127.0.0.1".to_string(),
    };
    info!("rabbitmq_uri={:?}", rabbitmq_uri);

    let options =
        ConnectionProperties::default().with_executor(tokio_executor_trait::Tokio::current());

    let connection = Connection::connect(&rabbitmq_uri, options).await.unwrap();

    // async block으로 소유권을 넘긴다
    let config_http = config.clone();
    let config_amqp = config.clone();

    let _result = join!(
        main_http(&config_http),
        main_amqp(&config_amqp, connection, settings.clone()),
    );
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
    config: &SdkConfig,
    mut connection: Connection,
    settings: Settings,
) -> Result<()> {
    // TODO: aws client 재사용하면 http 소켓 연결을 재사용할 수 있을거같은데
    // let client = aws_sdk_lambda::Client::new(config);
    // let lambda = MyAwsLambda::new(client);

    let channel = connection.create_channel().await.unwrap();

    for item in settings.queues.iter() {
        let queue_name = item.queue.clone();
        let function_name = item.function_name.clone();
        let config0 = config.clone();

        let _queue = channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        let consumer = channel
            .basic_consume(
                &queue_name,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        consumer.set_delegate(move |delivery: DeliveryResult| {
            let queue_name0 = queue_name.clone();
            let function_name0 = function_name.clone();
            let config1 = config0.clone();

            async move {
                let delivery = match delivery {
                    // Carries the delivery alongside its channel
                    Ok(Some(delivery)) => delivery,
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
                let event_bytes = event_text.into_bytes();

                let client = aws_sdk_lambda::Client::new(&config1);
                let lambda = MyAwsLambda::new(client);
                let _result = lambda.invoke(&function_name0, &event_bytes).await;

                info!("queue:{} = function:{}", queue_name0, function_name0);

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack send_webhook_event message");
            }
        });
    }

    Ok(())
}
