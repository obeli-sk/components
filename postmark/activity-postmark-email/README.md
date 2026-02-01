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

### Simple Email

```wit
send-simple(simple-email {
    from-email: "sender@yourdomain.com",
    from-name: some("Your Name"),
    to-email: "recipient@example.com",
    to-name: some("Recipient Name"),
    subject: "Hello from Obelisk!",
    text-body: some("Plain text version"),
    html-body: some("<h1>HTML version</h1>"),
    tag: some("welcome-email"),
})
```

### Full Email with Attachments

```wit
send(email-message {
    sender: email-address { email: "sender@yourdomain.com", name: some("Sender") },
    to: [email-address { email: "recipient@example.com", name: none }],
    cc: some([email-address { email: "cc@example.com", name: none }]),
    bcc: none,
    subject: "Document Attached",
    text-body: some("Please see attached."),
    html-body: some("<p>Please see attached.</p>"),
    reply-to: some(email-address { email: "reply@yourdomain.com", name: none }),
    headers: some([email-header { name: "X-Custom-Header", value: "custom-value" }]),
    attachments: some([
        attachment {
            name: "document.pdf",
            content: "base64-encoded-content-here",
            content-type: "application/pdf",
            content-id: none,
        }
    ]),
    tag: some("invoice"),
    metadata: some([metadata-entry { key: "order-id", value: "12345" }]),
    message-stream: some(outbound),
    track-opens: some(true),
    track-links: some("HtmlAndText"),
})
```

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
