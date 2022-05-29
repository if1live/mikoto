use anyhow::Result;
use aws_sdk_lambda::Region;
use aws_smithy_http::endpoint::Endpoint;
use aws_types::{credentials::SharedCredentialsProvider, Credentials, SdkConfig};
use std::env;
use std::sync::Arc;
use warp::hyper::Uri;

pub struct MyAwsConfig {}
impl MyAwsConfig {
    pub async fn new(origin: &str) -> aws_types::SdkConfig {
        match origin {
            "env" => MyAwsConfig::from_env(),
            "offline" => MyAwsConfig::from_offline(),
            _ => MyAwsConfig::from_offline(),
        }
    }

    pub fn from_offline() -> aws_types::SdkConfig {
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

    pub fn from_env() -> aws_types::SdkConfig {
        let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID").unwrap();
        let aws_access_secret_key = env::var("AWS_ACCESS_SECRET_KEY").unwrap();

        let credentials = Credentials::new(
            aws_access_key_id,
            aws_access_secret_key,
            None,
            None,
            "local",
        );

        let aws_region = env::var("AWS_REGION").unwrap();
        let region = Region::new(aws_region);

        let mut builder = SdkConfig::builder();
        builder.set_region(region);
        builder.set_credentials_provider(Some(SharedCredentialsProvider::new(credentials)));
        builder.build()
    }
}

#[derive(Clone)]
pub struct MyAwsLambda {
    client: aws_sdk_lambda::Client,
}
impl MyAwsLambda {
    pub fn new(client: aws_sdk_lambda::Client) -> Self {
        MyAwsLambda { client: client }
    }

    #[allow(dead_code)]
    pub async fn invoke(&self, function_name: &str, payload: &[u8]) -> Result<()> {
        let blob = aws_smithy_types::Blob::new(payload);
        let _response = self
            .client
            .invoke()
            .function_name(function_name)
            .payload(blob)
            .invocation_type(aws_sdk_lambda::model::InvocationType::Event)
            .send()
            .await?;
        Ok(())
    }
}
