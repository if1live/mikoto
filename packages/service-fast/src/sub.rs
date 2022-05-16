use lambda_runtime::{handler_fn, Context, Error};
use serde::Deserialize;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(dispatch);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn dispatch(event: Value, context: Context) -> Result<Value, Error> {
    handle_demo(event, context).await
}

#[derive(Deserialize)]
struct DemoEvent {
    key1: String,
    key2: String,
}
async fn handle_demo(event: Value, _: Context) -> Result<Value, Error> {
    let data: DemoEvent = serde_json::from_value(event).unwrap();
    Ok(json!({
        "first": data.key1,
        "second": data.key2,
    }))
}
