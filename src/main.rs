#![deny(warnings)]
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_lambda::Region;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::byte_stream::ByteStream;
use aws_smithy_http::endpoint::Endpoint;
use aws_types::{credentials::SharedCredentialsProvider, Credentials, SdkConfig};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{error, sync::Arc};
use warp::hyper::Uri;
// use warp::Filter;

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    s_val: String,
    i_val: i32,
    f_val: f32,
}

#[tokio::main]
async fn main() {
    main_http().await
}

async fn main_http() {
    // Match any request and return hello world!
    // let routes = warp::any().map(|| "Hello, World!");
    // warp::serve(routes)
    //     // ipv6 + ipv6 any addr
    //     .run(([0, 0, 0, 0, 0, 0, 0, 0], 8080))
    //     .await;

    // TODO: real aws
    let _region_config = build_region_config().await;

    let _offline_config = build_offline_config();
    println!("{:?}", &_offline_config);

    let _client = aws_sdk_lambda::Client::new(&_offline_config);
    let _result = invoke_lambda(&_client).await;

    println!("{:?}", &_result);
    return;
}

async fn invoke_lambda(client: &aws_sdk_lambda::Client) -> Result<(), Box<dyn error::Error>> {
    let function_name = "mikoto-sample-dev-commonDequeue";

    // TODO:
    let command = json!({
        "s_val": "Hello".to_string(),
        "i_val": 42,
        "f_val": 3.14,
    });

    let text = serde_json::to_string(&command)?;
    let _stream = ByteStream::new(SdkBody::from(text.as_str()));
    // let response = client
    //     .invoke_async()
    //     .function_name(function_name)
    //     .invoke_args(stream)
    //     .send()
    //     .await?;
    let response = client.invoke().function_name(function_name).send().await?;

    println!("Response from invoke: {:#?}", response);

    Ok(())
}


fn build_offline_config() -> aws_types::SdkConfig {
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

async fn build_region_config() -> aws_types::SdkConfig {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-northeast-1");
    aws_config::from_env().region(region_provider).load().await
}
