use lambda_http::{
    handler,
    http::{Response, StatusCode},
    Body, IntoResponse, Request, RequestExt,
};
use lambda_runtime::{Context, Error};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler(handle_http);
    lambda_runtime::run(func).await?;
    Ok(())
}

struct ResponseWrapper {
    data: Response<Body>,
}
impl IntoResponse for ResponseWrapper {
    fn into_response(self) -> Response<Body> {
        self.data
    }
}

async fn handle_http(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let name = request
        .query_string_parameters()
        .get("name")
        .unwrap_or_else(|| "stranger")
        .to_owned();

    let body = json!({ "hello": name });

    let status = if name == "403" {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::IM_A_TEAPOT
    };

    let response = Response::builder()
        .header("x-hello", "world")
        .status(status)
        .body(Body::from(body.to_string()));

    match response {
        Ok(data) => Ok(ResponseWrapper { data }),
        Err(error) => Err(error.into()),
    }
}
