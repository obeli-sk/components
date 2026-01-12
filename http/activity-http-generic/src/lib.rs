use crate::{
    generated::export,
    generated::exports::obelisk_components::generic_http::http::{
        self, Guest, Method as WitMethod, RequestBody, RequestError, ResponseBody,
    },
};
use wstd::{
    http::{Body, Client, Method, Request},
    runtime::block_on,
};

mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

struct Component;
export!(Component with_types_in generated);

impl Guest for Component {
    fn request(
        method: http::Method,
        url: String,
        headers: http::Headers,
        body: http::RequestBody,
    ) -> Result<http::Response, http::RequestError> {
        block_on(async { handle_request(method, url, headers, body).await })
    }
}

async fn handle_request(
    method: WitMethod,
    url: String,
    headers: http::Headers,
    body: RequestBody,
) -> Result<http::Response, RequestError> {
    // 1. Map Method
    let req_method = match method {
        WitMethod::Get => Method::GET,
        WitMethod::Post => Method::POST,
        WitMethod::Put => Method::PUT,
        WitMethod::Delete => Method::DELETE,
        WitMethod::Patch => Method::PATCH,
        WitMethod::Head => Method::HEAD,
        WitMethod::Options => Method::OPTIONS,
    };

    // 2. Extract Body Bytes from Variant
    let body_bytes = match body {
        RequestBody::Text(s) => s.into_bytes(),
        RequestBody::Binary(b) => b,
    };

    // 3. Build Request
    let mut builder = Request::builder().method(req_method).uri(&url);

    for (key, value) in headers {
        builder = builder.header(key, value);
    }

    let req_body = if body_bytes.is_empty() {
        Body::empty()
    } else {
        Body::from(body_bytes)
    };

    let request = builder
        .body(req_body)
        .map_err(|e| RequestError::InvalidUrl(format!("Failed to build request: {e}")))?;

    // 4. Send Request
    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| RequestError::NetworkError(e.to_string()))?;

    let status_code = response.status().as_u16();

    // 5. Extract Response Headers & Check Content-Type
    let mut res_headers = Vec::new();
    let mut is_text_content = false;

    for (key, value) in response.headers().iter() {
        let key_str = key.to_string();

        if key_str.eq_ignore_ascii_case("content-type") {
            let val_bytes = value.as_bytes();
            if val_bytes.starts_with(b"text/")
                || val_bytes.starts_with(b"application/json")
                || val_bytes.starts_with(b"application/xml")
                || val_bytes.starts_with(b"application/javascript")
            {
                is_text_content = true;
            }
        }

        let val_str = String::from_utf8_lossy(value.as_bytes()).into_owned();
        res_headers.push((key_str, val_str));
    }

    // 6. Process Response Body
    let mut response = response.into_body();
    let raw_bytes = Vec::from(
        response
            .contents()
            .await
            .map_err(|e| RequestError::IoError(e.to_string()))?,
    );

    let body_variant = if is_text_content {
        match String::from_utf8(raw_bytes) {
            Ok(text) => ResponseBody::Text(text),
            Err(e) => ResponseBody::Binary(e.into_bytes()),
        }
    } else {
        ResponseBody::Binary(raw_bytes)
    };

    Ok(http::Response {
        status_code,
        headers: res_headers,
        body: body_variant,
    })
}
