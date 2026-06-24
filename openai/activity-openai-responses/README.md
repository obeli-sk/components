# OpenAI Responses API Activity

A WASIp2 component that implements the [OpenAI Responses API](https://platform.openai.com/docs/api-reference/responses).
Designed to be used as an [Obelisk](https://obeli.sk/) activity.

It implements the [`api` WIT interface](wit/obelisk-components_openai-responses/openai-responses.wit) which provides:
- `create` - Create a response with structured input messages
- `create-simple` - Create a response with simple text input
- `get` - Retrieve a response by ID
- `delete` - Delete a response
- `list-responses` - List stored responses
- `list-input-items` - List input items for a response
- `cancel` - Cancel an in-progress response

## Prerequisites

An [OpenAI API key](https://platform.openai.com/api-keys) is required.
The key must be accessible as the `OPENAI_API_KEY` environment variable.

```sh
export OPENAI_API_KEY="sk-..."
```

## Building

```sh
just build
```

The component will be at `target/wasm32-wasip2/release_activity/activity_openai_responses.wasm`.

## WIT Interface

The component exports `obelisk-components:openai-responses/api@0.1.0` with the following key types:

### Error Handling

Errors use a variant type compatible with Obelisk's requirements:

```wit
variant api-error {
    /// For Obelisk compatibility - indicates timeout or trap in last retry
    execution-failed,
    /// Missing or invalid API key
    configuration-error(string),
    /// HTTP request failed (network, timeout, etc.)
    request-failed(string),
    /// OpenAI API returned an error
    api-error(api-error-details),
    /// Failed to parse response
    parse-error(string),
}
```

### Input Types

- `create-response-request` - Full request with structured messages, tools, reasoning config
- `simple-request` - Simplified request with just text input
- `input-message` - Message with role and content (text, images)
- `tool` - Function, web search, file search, code interpreter, or computer use tools

### Output Types

- `response` - Full response with output items, usage, status
- `output-item` - Message, function call, web search, file search, reasoning output
- `usage` - Token usage information

## Running with Obelisk

Build the activity and run Obelisk with appropriate configuration:

```sh
just build
obelisk server run --deployment ./obelisk-local.toml
```

Submit a simple request:

```sh
obelisk execution submit \
    -f .../api.create-simple \
    -- '{"model": "gpt-4o-mini", "input": "Hello, world!"}'
```

## WASI Imports

The component requires the following WASI interfaces:

- `wasi:http/outgoing-handler@0.2.9` - For HTTP requests to OpenAI
- `wasi:cli/environment@0.2.9` - For reading the API key from env vars
- `wasi:io/*@0.2.9` - Standard I/O interfaces
- `wasi:clocks/monotonic-clock@0.2.9` - For async runtime

## License

MIT
