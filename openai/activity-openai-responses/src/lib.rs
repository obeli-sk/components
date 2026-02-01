#![allow(unused)]

wit_bindgen::generate!({
    world: "any",
    path: "wit",
    generate_all,
});

mod client;

use exports::obelisk_components::openai_responses::api::*;

struct OpenAIResponses;

impl Guest for OpenAIResponses {
    fn create(request: CreateResponseRequest) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async {
            client::create_response(request).await
        })
    }

    fn create_simple(request: SimpleRequest) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async {
            client::create_simple_response(request).await
        })
    }

    fn get(response_id: String) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async {
            client::get_response(&response_id).await
        })
    }

    fn delete(response_id: String) -> Result<bool, ApiError> {
        wstd::runtime::block_on(async {
            client::delete_response(&response_id).await
        })
    }

    fn list_responses(
        limit: Option<u32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListResponsesResult, ApiError> {
        wstd::runtime::block_on(async {
            client::list_responses(limit, order, after, before).await
        })
    }

    fn list_input_items(
        response_id: String,
        limit: Option<u32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListInputItemsResult, ApiError> {
        wstd::runtime::block_on(async {
            client::list_input_items(&response_id, limit, order, after, before).await
        })
    }

    fn cancel(response_id: String) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async {
            client::cancel_response(&response_id).await
        })
    }
}

export!(OpenAIResponses);
