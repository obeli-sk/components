use crate::generated::{
    export,
    exports::obelisk_components::postmark_email::email::{
        ApiErrorDetail, Attachment, EmailAddress, EmailHeader, EmailMessage, Guest, MessageStream,
        MetadataEntry, SendError, SendResponse, SimpleEmail,
    },
};
use serde::{Deserialize, Serialize};
use wstd::{
    http::{Body, Client, Request},
    runtime::block_on,
};

mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

struct Component;
export!(Component with_types_in generated);

const POSTMARK_API_URL: &str = "https://api.postmarkapp.com/email";
const POSTMARK_SERVER_TOKEN: &str = "POSTMARK_SERVER_TOKEN";

/// Postmark API request body structure
#[derive(Serialize)]
struct PostmarkRequest {
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Cc", skip_serializing_if = "Option::is_none")]
    cc: Option<String>,
    #[serde(rename = "Bcc", skip_serializing_if = "Option::is_none")]
    bcc: Option<String>,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "TextBody", skip_serializing_if = "Option::is_none")]
    text_body: Option<String>,
    #[serde(rename = "HtmlBody", skip_serializing_if = "Option::is_none")]
    html_body: Option<String>,
    #[serde(rename = "ReplyTo", skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,
    #[serde(rename = "Headers", skip_serializing_if = "Option::is_none")]
    headers: Option<Vec<PostmarkHeader>>,
    #[serde(rename = "Attachments", skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<PostmarkAttachment>>,
    #[serde(rename = "Tag", skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(rename = "Metadata", skip_serializing_if = "Option::is_none")]
    metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "MessageStream", skip_serializing_if = "Option::is_none")]
    message_stream: Option<String>,
    #[serde(rename = "TrackOpens", skip_serializing_if = "Option::is_none")]
    track_opens: Option<bool>,
    #[serde(rename = "TrackLinks", skip_serializing_if = "Option::is_none")]
    track_links: Option<String>,
}

#[derive(Serialize)]
struct PostmarkHeader {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Serialize)]
struct PostmarkAttachment {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Content")]
    content: String,
    #[serde(rename = "ContentType")]
    content_type: String,
    #[serde(rename = "ContentId", skip_serializing_if = "Option::is_none")]
    content_id: Option<String>,
}

#[derive(Deserialize)]
struct PostmarkSuccessResponse {
    #[serde(rename = "MessageID")]
    message_id: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "SubmittedAt")]
    submitted_at: String,
}

#[derive(Deserialize)]
struct PostmarkErrorResponse {
    #[serde(rename = "ErrorCode")]
    error_code: u32,
    #[serde(rename = "Message")]
    message: String,
}

fn format_email_address(addr: &EmailAddress) -> String {
    match &addr.name {
        Some(name) => format!("{} <{}>", name, addr.email),
        None => addr.email.clone(),
    }
}

fn format_email_list(addrs: &[EmailAddress]) -> String {
    addrs
        .iter()
        .map(format_email_address)
        .collect::<Vec<_>>()
        .join(", ")
}

impl From<&Attachment> for PostmarkAttachment {
    fn from(att: &Attachment) -> Self {
        PostmarkAttachment {
            name: att.name.clone(),
            content: att.content.clone(),
            content_type: att.content_type.clone(),
            content_id: att.content_id.clone(),
        }
    }
}

impl From<&EmailHeader> for PostmarkHeader {
    fn from(h: &EmailHeader) -> Self {
        PostmarkHeader {
            name: h.name.clone(),
            value: h.value.clone(),
        }
    }
}

fn get_server_token() -> Result<String, SendError> {
    std::env::var(POSTMARK_SERVER_TOKEN).map_err(|_| {
        SendError::ConfigurationError(format!(
            "Environment variable {} is not set",
            POSTMARK_SERVER_TOKEN
        ))
    })
}

fn metadata_to_map(entries: &[MetadataEntry]) -> std::collections::HashMap<String, String> {
    entries
        .iter()
        .map(|e| (e.key.clone(), e.value.clone()))
        .collect()
}

fn message_stream_to_string(stream: &MessageStream) -> String {
    match stream {
        MessageStream::Outbound => "outbound".to_string(),
        MessageStream::Broadcast => "broadcast".to_string(),
    }
}

async fn send_request(body: PostmarkRequest) -> Result<SendResponse, SendError> {
    let server_token = get_server_token()?;
    let json_body = serde_json::to_string(&body)
        .map_err(|e| SendError::ValidationError(format!("Failed to serialize request: {}", e)))?;

    let req = Request::post(POSTMARK_API_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("X-Postmark-Server-Token", &server_token)
        .body(Body::from(json_body))
        .map_err(|e| SendError::RequestFailed(format!("Failed to build request: {}", e)))?;

    let resp = Client::new()
        .send(req)
        .await
        .map_err(|e| SendError::RequestFailed(format!("HTTP request failed: {}", e)))?;

    let status = resp.status().as_u16();

    let mut body = resp.into_body();
    let body_bytes = body
        .contents()
        .await
        .map_err(|e| SendError::RequestFailed(format!("Failed to read response body: {}", e)))?;

    let body_str = String::from_utf8_lossy(body_bytes);

    // Postmark returns 200 on success
    if status == 200 {
        let success: PostmarkSuccessResponse = serde_json::from_str(&body_str).map_err(|e| {
            SendError::RequestFailed(format!("Failed to parse success response: {}", e))
        })?;
        return Ok(SendResponse {
            message_id: success.message_id,
            to: success.to,
            submitted_at: success.submitted_at,
        });
    }

    // Try to parse Postmark error format
    if let Ok(error_response) = serde_json::from_str::<PostmarkErrorResponse>(&body_str) {
        return Err(SendError::ApiError(ApiErrorDetail {
            error_code: error_response.error_code,
            message: error_response.message,
        }));
    }

    // Return generic error if we can't parse the response
    Err(SendError::RequestFailed(format!(
        "Postmark API returned status {}: {}",
        status, body_str
    )))
}

impl Guest for Component {
    fn send_simple(email: SimpleEmail) -> Result<SendResponse, SendError> {
        // Validate that at least one content type is provided
        if email.text_body.is_none() && email.html_body.is_none() {
            return Err(SendError::ValidationError(
                "Either text_body or html_body must be provided".to_string(),
            ));
        }

        let from = match &email.from_name {
            Some(name) => format!("{} <{}>", name, email.from_email),
            None => email.from_email,
        };

        let to = match &email.to_name {
            Some(name) => format!("{} <{}>", name, email.to_email),
            None => email.to_email,
        };

        let request = PostmarkRequest {
            from,
            to,
            cc: None,
            bcc: None,
            subject: email.subject,
            text_body: email.text_body,
            html_body: email.html_body,
            reply_to: None,
            headers: None,
            attachments: None,
            tag: email.tag,
            metadata: None,
            message_stream: None,
            track_opens: None,
            track_links: None,
        };

        block_on(send_request(request))
    }

    fn send(message: EmailMessage) -> Result<SendResponse, SendError> {
        // Validate recipients
        if message.to.is_empty() {
            return Err(SendError::ValidationError(
                "At least one recipient is required".to_string(),
            ));
        }

        // Validate content
        if message.text_body.is_none() && message.html_body.is_none() {
            return Err(SendError::ValidationError(
                "Either text_body or html_body must be provided".to_string(),
            ));
        }

        let request = PostmarkRequest {
            from: format_email_address(&message.sender),
            to: format_email_list(&message.to),
            cc: message.cc.as_ref().map(|cc| format_email_list(cc)),
            bcc: message.bcc.as_ref().map(|bcc| format_email_list(bcc)),
            subject: message.subject,
            text_body: message.text_body,
            html_body: message.html_body,
            reply_to: message.reply_to.as_ref().map(format_email_address),
            headers: message
                .headers
                .as_ref()
                .map(|h| h.iter().map(PostmarkHeader::from).collect()),
            attachments: message
                .attachments
                .as_ref()
                .map(|a| a.iter().map(PostmarkAttachment::from).collect()),
            tag: message.tag,
            metadata: message.metadata.as_ref().map(|m| metadata_to_map(m)),
            message_stream: message
                .message_stream
                .as_ref()
                .map(message_stream_to_string),
            track_opens: message.track_opens,
            track_links: message.track_links,
        };

        block_on(send_request(request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENV_TOKEN: &str = "POSTMARK_SERVER_TOKEN";

    fn set_up() {
        let test_token = std::env::var(format!("TEST_{}", ENV_TOKEN))
            .expect("TEST_POSTMARK_SERVER_TOKEN must be set");
        unsafe { std::env::set_var(ENV_TOKEN, test_token) };
    }

    #[test]
    fn test_validation_no_content() {
        let email = SimpleEmail {
            from_email: "test@example.com".to_string(),
            from_name: None,
            to_email: "recipient@example.com".to_string(),
            to_name: None,
            subject: "Test".to_string(),
            text_body: None,
            html_body: None,
            tag: None,
        };

        let result = Component::send_simple(email);
        assert!(matches!(result, Err(SendError::ValidationError(_))));
    }

    #[test]
    fn test_validation_empty_recipients() {
        let message = EmailMessage {
            sender: EmailAddress {
                email: "test@example.com".to_string(),
                name: None,
            },
            to: vec![],
            cc: None,
            bcc: None,
            subject: "Test".to_string(),
            text_body: Some("Hello".to_string()),
            html_body: None,
            reply_to: None,
            headers: None,
            attachments: None,
            tag: None,
            metadata: None,
            message_stream: None,
            track_opens: None,
            track_links: None,
        };

        let result = Component::send(message);
        assert!(matches!(result, Err(SendError::ValidationError(_))));
    }

    #[test]
    fn test_validation_no_content_full() {
        let message = EmailMessage {
            sender: EmailAddress {
                email: "test@example.com".to_string(),
                name: None,
            },
            to: vec![EmailAddress {
                email: "recipient@example.com".to_string(),
                name: None,
            }],
            cc: None,
            bcc: None,
            subject: "Test".to_string(),
            text_body: None,
            html_body: None,
            reply_to: None,
            headers: None,
            attachments: None,
            tag: None,
            metadata: None,
            message_stream: None,
            track_opens: None,
            track_links: None,
        };

        let result = Component::send(message);
        assert!(matches!(result, Err(SendError::ValidationError(_))));
    }

    #[test]
    fn test_format_email_address() {
        let addr = EmailAddress {
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };
        assert_eq!(format_email_address(&addr), "Test User <test@example.com>");

        let addr_no_name = EmailAddress {
            email: "test@example.com".to_string(),
            name: None,
        };
        assert_eq!(format_email_address(&addr_no_name), "test@example.com");
    }

    #[test]
    #[ignore] // Integration test - requires API token and verified sender
    fn test_send_simple_email() {
        set_up();

        let email = SimpleEmail {
            from_email: "sender@yourdomain.com".to_string(), // Must be verified in Postmark
            from_name: Some("Test Sender".to_string()),
            to_email: "recipient@example.com".to_string(),
            to_name: Some("Test Recipient".to_string()),
            subject: "Test Email from Obelisk Postmark Activity".to_string(),
            text_body: Some(
                "This is a test email sent via the Obelisk Postmark activity.".to_string(),
            ),
            html_body: Some("<h1>Test Email</h1><p>This is a test email sent via the <strong>Obelisk Postmark activity</strong>.</p>".to_string()),
            tag: Some("test".to_string()),
        };

        let result = Component::send_simple(email);
        assert!(result.is_ok(), "Failed to send email: {:?}", result.err());
        let response = result.unwrap();
        assert!(
            !response.message_id.is_empty(),
            "Expected message_id in response"
        );
    }
}
