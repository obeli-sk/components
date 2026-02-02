# Postmark Email Activity

An Obelisk activity for sending emails via the [Postmark API](https://postmarkapp.com/developer/api/email-api).

## Features

- **Simple API**: `send-simple` for basic emails with minimal configuration
- **Full API**: `send` for advanced use cases with CC/BCC, attachments, metadata, etc.
- Supports plain text and HTML content
- File attachments with base64 encoding
- Custom headers and metadata for tracking
- Message streams (transactional vs broadcast)
- Open and link tracking options
- Returns Postmark message ID for tracking

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `POSTMARK_SERVER_TOKEN` | Yes | Your Postmark Server API Token |

## Setup

1. [Create a Postmark account](https://postmarkapp.com/)
2. Create a Server and get the Server API Token
3. [Verify your sender signature](https://account.postmarkapp.com/signatures) (domain or email)

## Usage Examples

First, start the Obelisk server:

```bash
export POSTMARK_SERVER_TOKEN="your-token"
obelisk server run --config ./obelisk-local.toml
```

Then submit executions using the CLI:

### Simple Email

```bash
obelisk client execution submit \
  obelisk-components:postmark-email/email.send-simple \
  '{
    "from_email": "sender@yourdomain.com",
    "from_name": "Your Name",
    "to_email": "recipient@example.com",
    "to_name": "Recipient Name",
    "subject": "Hello from Obelisk!",
    "text_body": "Plain text version",
    "html_body": "<h1>HTML version</h1>",
    "tag": "welcome-email"
  }'
```

### Full Email with Attachments

```bash
obelisk client execution submit \
  obelisk-components:postmark-email/email.send \
  '{
    "sender": { "email": "sender@yourdomain.com", "name": "Sender" },
    "to": [{ "email": "recipient@example.com", "name": null }],
    "cc": [{ "email": "cc@example.com", "name": null }],
    "bcc": null,
    "subject": "Document Attached",
    "text_body": "Please see attached.",
    "html_body": "<p>Please see attached.</p>",
    "reply_to": { "email": "reply@yourdomain.com", "name": null },
    "headers": [{ "name": "X-Custom-Header", "value": "custom-value" }],
    "attachments": [
      {
        "name": "document.pdf",
        "content": "base64-encoded-content-here",
        "content_type": "application/pdf",
        "content_id": null
      }
    ],
    "tag": "invoice",
    "metadata": [{ "key": "order-id", "value": "12345" }],
    "message_stream": "outbound",
    "track_opens": true,
    "track_links": "HtmlAndText"
  }'

## Error Handling

| Error Variant | Description |
|---------------|-------------|
| `execution-failed` | Reserved for Obelisk (timeouts, traps) |
| `configuration-error` | Missing or invalid API token |
| `validation-error` | Invalid email format or missing required fields |
| `api-error` | Postmark API returned an error with code and message |
| `request-failed` | Network or HTTP error |

## Postmark Error Codes

Common error codes from Postmark:
- `10` - Bad or missing API token
- `300` - Invalid email request
- `400` - Sender signature not found
- `405` - Not allowed to send
- `406` - Inactive recipient

See [Postmark API Error Codes](https://postmarkapp.com/developer/api/overview#error-codes) for full list.

## Building

```bash
cargo build --release --profile release_activity
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires valid Postmark token and verified sender)
export TEST_POSTMARK_SERVER_TOKEN="your-token"
cargo test -- --ignored
```

## Local Development with Obelisk

```bash
export POSTMARK_SERVER_TOKEN="your-token"
cargo build --release --profile release_activity
obelisk server run --config ./obelisk-local.toml
```
