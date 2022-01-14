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

async fn handle_http(request: Request, context: Context) -> Result<impl IntoResponse, Error> {
    let response = match request.uri().path() {
        "/" => handle_foo(request, context).await,
        _ => handle_not_found(request, context).await,
    };

    match response {
        Ok(data) => Ok(ResponseWrapper { data }),
        Err(error) => Err(error.into()),
    }
}

async fn handle_foo(request: Request, _: Context) -> http::Result<Response<Body>> {
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
    response
}

async fn handle_not_found(_: Request, _: Context) -> http::Result<Response<Body>> {
    let body = json!({"message": "not found"});
    let response = Response::builder().body(Body::from(body.to_string()));
    response
}
