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
        body: Option<http::RequestBody>,
    ) -> Result<http::Response, http::RequestError> {
        block_on(async { handle_request(method, url, headers, body).await })
    }
}

async fn handle_request(
    method: WitMethod,
    url: String,
    headers: http::Headers,
    body: Option<RequestBody>,
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

    // 2. Extract Body Bytes (Handle Option)
    let body_bytes = match body {
        Some(RequestBody::Text(s)) => s.into_bytes(),
        Some(RequestBody::Binary(b)) => b,
        None => Vec::new(),
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

    // If body is empty, return None
    let body_option = if raw_bytes.is_empty() {
        None
    } else if is_text_content {
        match String::from_utf8(raw_bytes) {
            Ok(text) => Some(ResponseBody::Text(text)),
            Err(e) => Some(ResponseBody::Binary(e.into_bytes())),
        }
    } else {
        Some(ResponseBody::Binary(raw_bytes))
    };

    Ok(http::Response {
        status_code,
        headers: res_headers,
        body: body_option,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENV_TEST_BASE_URL: &str = "TEST_HTTP_BASE_URL";

    fn get_test_url(path: &str) -> String {
        let base =
            std::env::var(ENV_TEST_BASE_URL).expect("TEST_HTTP_BASE_URL must be set for e2e tests");
        format!("{}{}", base.trim_end_matches('/'), path)
    }

    #[test]
    #[ignore]
    fn test_get_request() {
        let url = get_test_url("/echo");
        let result = Component::request(WitMethod::Get, url, vec![], None);
        let response = result.expect("GET request should succeed");

        assert_eq!(response.status_code, 200);
        match response.body {
            Some(ResponseBody::Text(body)) => {
                assert!(
                    body.contains("GET"),
                    "Expected body to contain GET method, got: {}",
                    body
                );
            }
            other => panic!("Expected text response, got: {:?}", other),
        }
    }

    #[test]
    #[ignore]
    fn test_post_request_with_body() {
        let url = get_test_url("/echo");
        let body = RequestBody::Text("{\"test\": \"data\"}".to_string());
        let headers = vec![("Content-Type".to_string(), "application/json".to_string())];

        let result = Component::request(WitMethod::Post, url, headers, Some(body));
        let response = result.expect("POST request should succeed");

        assert_eq!(response.status_code, 200);
        match response.body {
            Some(ResponseBody::Text(body)) => {
                assert!(
                    body.contains("POST"),
                    "Expected body to contain POST method, got: {}",
                    body
                );
                assert!(
                    body.contains("test"),
                    "Expected body to contain test data, got: {}",
                    body
                );
            }
            other => panic!("Expected text response, got: {:?}", other),
        }
    }

    #[test]
    #[ignore]
    fn test_404_response() {
        let url = get_test_url("/status/404");
        let result = Component::request(WitMethod::Get, url, vec![], None);
        let response = result.expect("Request should complete even with 404");

        assert_eq!(response.status_code, 404);
    }

    #[test]
    #[ignore]
    fn test_empty_response() {
        let url = get_test_url("/empty");
        let result = Component::request(WitMethod::Get, url, vec![], None);
        let response = result.expect("Request should succeed");

        assert_eq!(response.status_code, 204);
        assert!(response.body.is_none());
    }

    #[test]
    #[ignore]
    fn test_binary_response() {
        let url = get_test_url("/binary");
        let result = Component::request(WitMethod::Get, url, vec![], None);
        let response = result.expect("Request should succeed");

        assert_eq!(response.status_code, 200);
        assert!(matches!(response.body, Some(ResponseBody::Binary(_))));
    }

    #[test]
    #[ignore]
    fn test_put_request() {
        let url = get_test_url("/echo");
        let body = RequestBody::Text("update data".to_string());

        let result = Component::request(WitMethod::Put, url, vec![], Some(body));
        let response = result.expect("PUT request should succeed");

        assert_eq!(response.status_code, 200);
        match response.body {
            Some(ResponseBody::Text(body)) => {
                assert!(
                    body.contains("PUT"),
                    "Expected body to contain PUT method, got: {}",
                    body
                );
            }
            other => panic!("Expected text response, got: {:?}", other),
        }
    }

    #[test]
    #[ignore]
    fn test_delete_request() {
        let url = get_test_url("/echo");

        let result = Component::request(WitMethod::Delete, url, vec![], None);
        let response = result.expect("DELETE request should succeed");

        assert_eq!(response.status_code, 200);
        match response.body {
            Some(ResponseBody::Text(body)) => {
                assert!(
                    body.contains("DELETE"),
                    "Expected body to contain DELETE method, got: {}",
                    body
                );
            }
            other => panic!("Expected text response, got: {:?}", other),
        }
    }
}
