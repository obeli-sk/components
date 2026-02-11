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

First, start the Obelisk server:

```bash
export SENDGRID_API_KEY="SG.your-api-key"
obelisk server run --config ./obelisk-local.toml
```

Then submit executions using the CLI:

### Simple Email

```bash
obelisk execution submit --follow \
  obelisk-components:sendgrid-email/email.send-simple \
  -- '{
    "from_email": "sender@yourdomain.com",
    "from_name": "Your Name",
    "to_email": "recipient@example.com",
    "to_name": "Recipient Name",
    "subject": "Hello from Obelisk!",
    "text_content": "Plain text version",
    "html_content": "<h1>HTML version</h1>"
  }'
```

### Full Email with Attachments

```bash
obelisk execution submit --follow \
  obelisk-components:sendgrid-email/email.send \
  -- '{
    "sender": { "email": "sender@yourdomain.com", "name": "Sender" },
    "subject": "Document Attached",
    "personalizations": [
      {
        "to": [{ "email": "recipient@example.com", "name": null }],
        "cc": [{ "email": "cc@example.com", "name": null }],
        "bcc": null,
        "subject": null
      }
    ],
    "content": [
      { "mime_type": "text/plain", "value": "Please see attached." },
      { "mime_type": "text/html", "value": "<p>Please see attached.</p>" }
    ],
    "reply_to": { "email": "reply@yourdomain.com", "name": null },
    "attachments": [
      {
        "content": "base64-encoded-content-here",
        "filename": "document.pdf",
        "mime_type": "application/pdf",
        "disposition": "attachment",
        "content_id": null
      }
    ]
  }'

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
