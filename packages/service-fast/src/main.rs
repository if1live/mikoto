use hyper::service::{make_service_fn, service_fn};
use hyper::{
    http::{Error, StatusCode},
    Body, Request, Response, Server,
};
use lambda_http::{handler, IntoResponse};
use serde_json::json;
use std::env;
use std::{convert::Infallible, net::SocketAddr};

#[tokio::main]
async fn main() {
    let key = "AWS_LAMBDA_FUNCTION_NAME";
    match env::var(key) {
        Ok(_) => {
            println!("mode: lambda");
            let _ = main_lambda().await;
            ()
        }
        Err(_) => {
            println!("mode: local");
            let _ = main_local().await;
            ()
        }
    }
}

async fn main_local() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_foo)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn main_lambda() -> Result<(), lambda_runtime::Error> {
    let func = handler(handle_http);
    lambda_runtime::run(func).await?;
    Ok(())
}

struct ResponseWrapper {
    body: String,
}
impl IntoResponse for ResponseWrapper {
    fn into_response(self) -> lambda_http::Response<lambda_http::Body> {
        let body = lambda_http::Body::from(self.body);
        lambda_http::Response::new(body)
    }
}

async fn handle_http(
    request: lambda_http::Request,
    _context: lambda_runtime::Context,
) -> Result<impl lambda_http::IntoResponse, lambda_runtime::Error> {
    // let response = match request.uri().path() {
    //     "/" => handle_foo(request, context).await,
    //     _ => handle_not_found(request, context).await,
    // };

    // match response {
    //     Ok(data) => Ok(ResponseWrapper { data }),
    //     Err(error) => Err(error.into()),
    // }
    let (parts, body_lambda) = request.into_parts();
    let body_hyper = match body_lambda {
        lambda_http::Body::Text(text) => Body::from(text.clone()),
        _ => Body::from(""),
    };
    let request_hyper = Request::from_parts(parts, body_hyper);
    let result = handle_foo(request_hyper).await;
    match result {
        Ok(resp) => {
            let bytes = hyper::body::to_bytes(resp.into_body()).await.unwrap();
            let body = String::from_utf8(bytes.to_vec()).unwrap();
            Ok(ResponseWrapper { body })
        }
        Err(error) => Err(error.into()),
    }
}

async fn handle_foo(request: Request<Body>) -> Result<Response<Body>, Error> {
    let uri = request.uri().clone();

    println!("method: {}", request.method());
    println!("uri: {}", uri);

    let bytes = match hyper::body::to_bytes(request.into_body()).await {
        Ok(data) => data,
        Err(_) => hyper::body::Bytes::new(),
    };
    println!("body: {}", String::from_utf8(bytes.to_vec()).unwrap());

    // let name = request
    //     .query_string_parameters()
    //     .get("name")
    //     .unwrap_or_else(|| "stranger")
    //     .to_owned();
    let name = uri.query();

    let body = json!({ "hello": name });

    let status = if name.unwrap_or("empty") == "403" {
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

// async fn handle_not_found(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//     let body = json!({"message": "not found"});
//     Ok(Response::new(body.to_string().into()))
// }
