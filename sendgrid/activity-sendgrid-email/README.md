# SendGrid Email Activity

An Obelisk activity for sending emails via the [SendGrid API](https://docs.sendgrid.com/api-reference/mail-send/mail-send).

## Features

- **Simple API**: `send-simple` for basic emails with minimal configuration
- **Full API**: `send` for advanced use cases with personalizations, CC/BCC, attachments, etc.
- Supports plain text and HTML content
- Supports file attachments
- Returns SendGrid message ID for tracking

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `SENDGRID_API_KEY` | Yes | Your SendGrid API key (starts with `SG.`) |

## Setup

1. [Create a SendGrid account](https://signup.sendgrid.com/)
2. [Create an API key](https://app.sendgrid.com/settings/api_keys) with "Mail Send" permission
3. [Verify your sender identity](https://app.sendgrid.com/settings/sender_auth) (domain or single sender)

## Usage Examples

### Simple Email

```wit
send-simple(simple-email {
    from-email: "sender@yourdomain.com",
    from-name: some("Your Name"),
    to-email: "recipient@example.com",
    to-name: some("Recipient Name"),
    subject: "Hello from Obelisk!",
    text-content: some("Plain text version"),
    html-content: some("<h1>HTML version</h1>"),
})
```

### Full Email with Attachments

```wit
send(email-message {
    sender: email-address { email: "sender@yourdomain.com", name: some("Sender") },
    subject: "Document Attached",
    personalizations: [
        personalization {
            to: [email-address { email: "recipient@example.com", name: none }],
            cc: some([email-address { email: "cc@example.com", name: none }]),
            bcc: none,
            subject: none,
        }
    ],
    content: [
        content { mime-type: "text/plain", value: "Please see attached." },
        content { mime-type: "text/html", value: "<p>Please see attached.</p>" },
    ],
    reply-to: some(email-address { email: "reply@yourdomain.com", name: none }),
    attachments: some([
        attachment {
            content: "base64-encoded-content-here",
            filename: "document.pdf",
            mime-type: some("application/pdf"),
            disposition: some("attachment"),
            content-id: none,
        }
    ]),
})
```

## Error Handling

| Error Variant | Description |
|---------------|-------------|
| `execution-failed` | Reserved for Obelisk (timeouts, traps) |
| `configuration-error` | Missing or invalid API key |
| `validation-error` | Invalid email format or missing required fields |
| `api-error` | SendGrid API returned an error with details |
| `request-failed` | Network or HTTP error |

## Building

```bash
cargo build --release --profile release_activity
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires valid SendGrid API key and verified sender)
export TEST_SENDGRID_API_KEY="SG.your-api-key"
cargo test -- --ignored
```

## Local Development with Obelisk

```bash
export SENDGRID_API_KEY="SG.your-api-key"
cargo build --release --profile release_activity
obelisk server run --config ./obelisk-local.toml
```
