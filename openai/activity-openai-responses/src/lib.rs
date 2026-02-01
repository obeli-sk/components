#![allow(unused)]

mod client;
mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

use generated::export;

use generated::exports::obelisk_components::openai_responses::api::*;

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

struct OpenAIResponses;

impl Guest for OpenAIResponses {
    fn create(request: CreateResponseRequest) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async { client::create_response(request).await })
    }

    fn create_simple(request: SimpleRequest) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async { client::create_simple_response(request).await })
    }

    fn get(response_id: String) -> Result<Response, ApiError> {
        wstd::runtime::block_on(async { client::get_response(&response_id).await })
    }

    fn delete(response_id: String) -> Result<bool, ApiError> {
        wstd::runtime::block_on(async { client::delete_response(&response_id).await })
    }

    fn list_responses(
        limit: Option<u32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListResponsesResult, ApiError> {
        wstd::runtime::block_on(async { client::list_responses(limit, order, after, before).await })
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
        wstd::runtime::block_on(async { client::cancel_response(&response_id).await })
    }
}

export!(OpenAIResponses with_types_in generated);

#[cfg(test)]
mod tests {
    use super::*;
    use generated::obelisk_components::openai_responses::types::{ResponseStatus, Role};

    fn set_up() {
        let test_token = std::env::var(format!("TEST_{ENV_OPENAI_API_KEY}")).unwrap_or_else(|_| {
            panic!("TEST_{ENV_OPENAI_API_KEY} must be set as an environment variable")
        });
        unsafe { std::env::set_var(ENV_OPENAI_API_KEY, test_token) };
    }

    #[test]
    #[ignore]
    fn create_simple_should_succeed() {
        set_up();

        let request = SimpleRequest {
            model: "gpt-4o-mini".to_string(),
            input: "Say 'hello test' and nothing else.".to_string(),
            instructions: None,
            max_output_tokens: Some(50),
            temperature: Some(0.0),
            tools: None,
        };

        let result = OpenAIResponses::create_simple(request);
        let response = result.expect("create_simple should succeed");

        assert_eq!(response.status, ResponseStatus::Completed);
        assert!(!response.output.is_empty(), "response should have output");
        println!("Response ID: {}", response.id);
        println!("Model: {}", response.model);
        println!("Output items: {}", response.output.len());
    }

    #[test]
    #[ignore]
    fn create_with_message_should_succeed() {
        use crate::generated::obelisk_components::openai_responses::types::{
            InputContent, InputMessage,
        };

        set_up();

        let request = CreateResponseRequest {
            model: "gpt-4o-mini".to_string(),
            input: vec![InputMessage {
                role: Role::User,
                content: vec![InputContent::Text(
                    "What is 2+2? Reply with just the number.".to_string(),
                )],
            }],
            instructions: Some("You are a calculator. Only respond with numbers.".to_string()),
            max_output_tokens: Some(50),
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            reasoning: None,
            store: None,
            temperature: Some(0.0),
            text: None,
            tool_choice: None,
            tools: None,
            top_p: None,
            truncation: None,
            user: None,
        };

        let result = OpenAIResponses::create(request);
        let response = result.expect("create should succeed");

        assert_eq!(response.status, ResponseStatus::Completed);
        println!("Response ID: {}", response.id);
        println!("Output items: {}", response.output.len());
    }

    #[test]
    fn missing_api_key_should_fail() {
        // Don't set up - leave API key unset
        unsafe { std::env::remove_var(ENV_OPENAI_API_KEY) };

        let request = SimpleRequest {
            model: "gpt-4o-mini".to_string(),
            input: "test".to_string(),
            instructions: None,
            max_output_tokens: None,
            temperature: None,
            tools: None,
        };

        let result = OpenAIResponses::create_simple(request);
        assert!(result.is_err(), "should fail without API key");

        match result.unwrap_err() {
            ApiError::ConfigurationError(msg) => {
                assert!(
                    msg.contains("OPENAI_API_KEY"),
                    "error should mention API key"
                );
            }
            other => panic!("expected ConfigurationError, got {:?}", other),
        }
    }
}
