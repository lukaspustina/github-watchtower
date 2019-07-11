use lambda_http::{http, lambda, Body, IntoResponse, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};
use log::debug;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    lambda!(lambda_handler);

    Ok(())
}

fn lambda_handler(request: Request, ctx: Context) -> Result<impl IntoResponse, HandlerError> {
    handle(request, ctx).map_err(|e| HandlerError::from(e.to_string().as_str()))
}

fn handle(request: Request, _ctx: Context) -> Result<impl IntoResponse, Box<dyn Error>> {
    debug!("Request headers: {:#?}", request.headers());
    debug!("Request body: {:#?}", request.body());
    debug!("Request payload: {:#?}", request.payload::<String>());
    debug!("Request context: {:#?}", request.request_context());

    match request
        .headers()
        .get(http::header::CONTENT_TYPE)
        .map(|x| x.to_str())
    {
        Some(Ok("application/json")) => {}
        _ => {
            return Ok(Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body(r#"{"message":"Invalid content type. Please only use JSON."}"#.into())?)
        }
    }

    let body = match request.body() {
        Body::Empty => "empty".to_string(),
        Body::Text(text) => text.to_string(),
        Body::Binary(_) => "binary".to_string(),
    };

    let r = Response::builder().body(body)?;

    Ok(r)
}
