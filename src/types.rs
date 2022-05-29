use aws_lambda_events::event::rabbitmq::{RabbitMqBasicProperties, RabbitMqEvent, RabbitMqMessage};
use lapin::message::Delivery;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueDefinition {
    pub queue: String,
    pub function_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MikotoDefinition {
    pub queues: Vec<QueueDefinition>,
}

pub struct MyRabbitEvent {}
impl MyRabbitEvent {
    pub fn to_event(delivery: &Delivery) -> RabbitMqEvent {
        // TODO: 이벤트 규격이 람다 규격과 비슷하도록 만들기
        let body_str = String::from_utf8_lossy(&delivery.data);

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
