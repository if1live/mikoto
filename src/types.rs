use aws_lambda_events::event::rabbitmq::{RabbitMqBasicProperties, RabbitMqEvent, RabbitMqMessage};
use chrono::{
    prelude::{DateTime, NaiveDateTime},
    Utc,
};
use config::{Config, ConfigError, File};
use lapin::{message::Delivery, types::ShortString};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct QueueDefinition {
    pub queue: String,
    pub function_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub queues: Vec<QueueDefinition>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("mikoto"))
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}

fn unwrap_string(input: &Option<ShortString>) -> Option<String> {
    match input {
        Some(s) => Some(s.to_string()),
        None => None,
    }
}

pub struct MyRabbitEvent {}
impl MyRabbitEvent {
    pub fn to_event(delivery: &Delivery) -> RabbitMqEvent {
        let body_str = String::from_utf8_lossy(&delivery.data);

        let field_table = delivery.properties.headers().clone().unwrap();
        let mut headers = HashMap::new();
        for (key, value) in field_table.into_iter() {
            let key = key.to_string();
            // TODO: 안돌아가면 뜯어서 타입 맞추기
            headers.insert(key, json!(value));
        }

        let timestamp = delivery.properties.timestamp().clone().map(|millis| {
            // https://stackoverflow.com/a/50072164
            let secs: i64 = (millis / 1000).try_into().unwrap();
            let usecs: u32 = (millis % 1000).try_into().unwrap();
            let naive = NaiveDateTime::from_timestamp(secs, usecs);
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            // Jan 1, 1970, 12:33:41 AM
            datetime.to_rfc2822()
        });

        let property = RabbitMqBasicProperties {
            content_type: unwrap_string(delivery.properties.content_type()),
            content_encoding: unwrap_string(delivery.properties.content_encoding()),
            headers,
            delivery_mode: delivery.properties.delivery_mode().unwrap_or(0),
            priority: delivery.properties.priority().unwrap_or(0),
            correlation_id: unwrap_string(delivery.properties.correlation_id()),
            reply_to: unwrap_string(delivery.properties.reply_to()),
            expiration: unwrap_string(delivery.properties.expiration()),
            message_id: unwrap_string(delivery.properties.message_id()),
            timestamp,
            // type과 제일 비슷해보이길래 kind를 연결
            type_: unwrap_string(delivery.properties.kind()),
            user_id: unwrap_string(delivery.properties.user_id()),
            app_id: unwrap_string(delivery.properties.app_id()),
            cluster_id: unwrap_string(delivery.properties.cluster_id()),
            body_size: body_str.len() as u64,
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
                (delivery.routing_key.to_string(), vec![message]),
            ]),
        };

        event
    }

    pub fn to_string(delivery: &Delivery) -> String {
        let event_json = json!(Self::to_event(delivery));
        event_json.to_string()
    }
}
