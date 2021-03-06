use anyhow::Result;
use aws_sdk_lambda::{Region, RetryConfig};
use aws_smithy_http::endpoint::Endpoint;
use aws_smithy_types::timeout;
use aws_smithy_types::tristate::TriState;
use aws_types::{credentials::SharedCredentialsProvider, Credentials, SdkConfig};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use warp::hyper::Uri;

pub struct MyAwsConfig {}
impl MyAwsConfig {
    pub fn new() -> Result<aws_types::SdkConfig> {
        let aws_region = env::var("AWS_REGION")?;
        let region = Region::new(aws_region);

        let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID")?;
        let aws_access_secret_key = env::var("AWS_ACCESS_SECRET_KEY")?;

        let credentials = Credentials::new(
            aws_access_key_id,
            aws_access_secret_key,
            None,
            None,
            "local",
        );

        let aws_url = match env::var("AWS_URL") {
            Ok(url) => match url.parse::<Uri>() {
                Ok(uri) => Some(uri),
                Err(_) => None,
            },
            Err(_) => None,
        };

        let retry_config = RetryConfig::new().with_max_attempts(1);

        let api_timeout_config = timeout::Api::new()
            .with_call_attempt_timeout(TriState::Set(Duration::from_secs(2)))
            .with_call_timeout(TriState::Set(Duration::from_secs(5)));
        let timeout_config = timeout::Config::new().with_api_timeouts(api_timeout_config);

        let mut builder = SdkConfig::builder();

        builder.set_region(region);
        builder.set_credentials_provider(Some(SharedCredentialsProvider::new(credentials)));

        if let Some(uri) = aws_url {
            let endpoint = Endpoint::immutable(uri);
            builder.set_endpoint_resolver(Some(Arc::new(endpoint)));
        }

        builder.set_retry_config(Some(retry_config));
        builder.set_timeout_config(Some(timeout_config));
        builder.set_sleep_impl(None);

        let config = builder.build();
        Ok(config)
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct MyAwsLambda {
    client: aws_sdk_lambda::Client,
}
impl MyAwsLambda {
    #[allow(unused)]
    pub fn new(client: aws_sdk_lambda::Client) -> Self {
        MyAwsLambda { client: client }
    }

    #[allow(unused)]
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
