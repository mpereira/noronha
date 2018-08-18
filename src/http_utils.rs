use std::str;

use actix_web::{
    AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse,
};
use bytes::Bytes;
use futures::future::Future;
use serde_json;

pub fn json_body(content: &serde_json::Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(content).unwrap())
}

pub fn json_error(error: serde_json::Error) -> String {
    json_body(&json!({ "error": format!("{:?}", error) }))
}

pub fn make_handler_for_request_with_body(
    handler: &'static for<'r> Fn(&HttpRequest, serde_json::Value)
        -> Result<HttpResponse, Error>,
) -> Box<Fn(HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>>> {
    Box::new(move |request: HttpRequest| {
        request
            .body()
            .from_err()
            .and_then(move |bytes: Bytes| -> Result<HttpResponse, Error> {
                let body: serde_json::Value = serde_json::from_str(
                    str::from_utf8(&bytes).unwrap(),
                ).unwrap_or(json!(null));
                handler(&request, body)
            })
            .responder()
    })
}
