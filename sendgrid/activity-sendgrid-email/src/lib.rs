use crate::generated::{
    export,
    exports::obelisk_components::sendgrid_email::email::{
        Attachment, Content, EmailAddress, EmailMessage, ErrorDetail, Guest, Personalization,
        SendError, SendResponse, SimpleEmail,
    },
};
use serde::Serialize;
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

const SENDGRID_API_URL: &str = "https://api.sendgrid.com/v3/mail/send";
const SENDGRID_API_KEY: &str = "SENDGRID_API_KEY";

/// SendGrid API request body structure
#[derive(Serialize)]
struct SendGridRequest {
    personalizations: Vec<SendGridPersonalization>,
    from: SendGridEmail,
    subject: String,
    content: Vec<SendGridContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<SendGridEmail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<SendGridAttachment>>,
}

#[derive(Serialize)]
struct SendGridPersonalization {
    to: Vec<SendGridEmail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cc: Option<Vec<SendGridEmail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bcc: Option<Vec<SendGridEmail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
}

#[derive(Serialize)]
struct SendGridEmail {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize)]
struct SendGridContent {
    #[serde(rename = "type")]
    mime_type: String,
    value: String,
}

#[derive(Serialize)]
struct SendGridAttachment {
    content: String,
    filename: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_id: Option<String>,
}

impl From<&EmailAddress> for SendGridEmail {
    fn from(addr: &EmailAddress) -> Self {
        SendGridEmail {
            email: addr.email.clone(),
            name: addr.name.clone(),
        }
    }
}

impl From<&Content> for SendGridContent {
    fn from(content: &Content) -> Self {
        SendGridContent {
            mime_type: content.mime_type.clone(),
            value: content.value.clone(),
        }
    }
}

impl From<&Attachment> for SendGridAttachment {
    fn from(att: &Attachment) -> Self {
        SendGridAttachment {
            content: att.content.clone(),
            filename: att.filename.clone(),
            mime_type: att.mime_type.clone(),
            disposition: att.disposition.clone(),
            content_id: att.content_id.clone(),
        }
    }
}

impl From<&Personalization> for SendGridPersonalization {
    fn from(p: &Personalization) -> Self {
        SendGridPersonalization {
            to: p.to.iter().map(SendGridEmail::from).collect(),
            cc: p
                .cc
                .as_ref()
                .map(|cc| cc.iter().map(SendGridEmail::from).collect()),
            bcc: p
                .bcc
                .as_ref()
                .map(|bcc| bcc.iter().map(SendGridEmail::from).collect()),
            subject: p.subject.clone(),
        }
    }
}

fn get_api_key() -> Result<String, SendError> {
    std::env::var(SENDGRID_API_KEY).map_err(|_| {
        SendError::ConfigurationError(format!(
            "Environment variable {} is not set",
            SENDGRID_API_KEY
        ))
    })
}

async fn send_request(body: SendGridRequest) -> Result<SendResponse, SendError> {
    let api_key = get_api_key()?;
    let json_body = serde_json::to_string(&body)
        .map_err(|e| SendError::ValidationError(format!("Failed to serialize request: {}", e)))?;

    let req = Request::post(SENDGRID_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json_body))
        .map_err(|e| SendError::RequestFailed(format!("Failed to build request: {}", e)))?;

    let resp = Client::new()
        .send(req)
        .await
        .map_err(|e| SendError::RequestFailed(format!("HTTP request failed: {}", e)))?;

    let status = resp.status().as_u16();

    // Extract message ID from headers before consuming response
    let message_id = resp
        .headers()
        .get("x-message-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // SendGrid returns 202 Accepted on success
    if status == 202 {
        return Ok(SendResponse { message_id });
    }

    // Parse error response
    let mut body = resp.into_body();
    let body_bytes = body
        .contents()
        .await
        .map_err(|e| SendError::RequestFailed(format!("Failed to read response body: {}", e)))?;

    let body_str = String::from_utf8_lossy(body_bytes);

    // Try to parse SendGrid error format
    if let Ok(error_response) = serde_json::from_str::<SendGridErrorResponse>(&body_str) {
        let errors: Vec<ErrorDetail> = error_response
            .errors
            .into_iter()
            .map(|e| ErrorDetail {
                message: e.message,
                field: e.field,
            })
            .collect();
        return Err(SendError::ApiError(errors));
    }

    // Return generic error if we can't parse the response
    Err(SendError::RequestFailed(format!(
        "SendGrid API returned status {}: {}",
        status, body_str
    )))
}

#[derive(serde::Deserialize)]
struct SendGridErrorResponse {
    errors: Vec<SendGridError>,
}

#[derive(serde::Deserialize)]
struct SendGridError {
    message: String,
    field: Option<String>,
}

impl Guest for Component {
    fn send_simple(email: SimpleEmail) -> Result<SendResponse, SendError> {
        // Validate that at least one content type is provided
        if email.text_content.is_none() && email.html_content.is_none() {
            return Err(SendError::ValidationError(
                "Either text_content or html_content must be provided".to_string(),
            ));
        }

        let mut content = Vec::new();
        if let Some(text) = &email.text_content {
            content.push(SendGridContent {
                mime_type: "text/plain".to_string(),
                value: text.clone(),
            });
        }
        if let Some(html) = &email.html_content {
            content.push(SendGridContent {
                mime_type: "text/html".to_string(),
                value: html.clone(),
            });
        }

        let request = SendGridRequest {
            personalizations: vec![SendGridPersonalization {
                to: vec![SendGridEmail {
                    email: email.to_email,
                    name: email.to_name,
                }],
                cc: None,
                bcc: None,
                subject: None,
            }],
            from: SendGridEmail {
                email: email.from_email,
                name: email.from_name,
            },
            subject: email.subject,
            content,
            reply_to: None,
            attachments: None,
        };

        block_on(send_request(request))
    }

    fn send(message: EmailMessage) -> Result<SendResponse, SendError> {
        // Validate personalizations
        if message.personalizations.is_empty() {
            return Err(SendError::ValidationError(
                "At least one personalization is required".to_string(),
            ));
        }

        for p in &message.personalizations {
            if p.to.is_empty() {
                return Err(SendError::ValidationError(
                    "Each personalization must have at least one recipient".to_string(),
                ));
            }
        }

        // Validate content
        if message.content.is_empty() {
            return Err(SendError::ValidationError(
                "At least one content block is required".to_string(),
            ));
        }

        let request = SendGridRequest {
            personalizations: message
                .personalizations
                .iter()
                .map(SendGridPersonalization::from)
                .collect(),
            from: SendGridEmail::from(&message.sender),
            subject: message.subject,
            content: message.content.iter().map(SendGridContent::from).collect(),
            reply_to: message.reply_to.as_ref().map(SendGridEmail::from),
            attachments: message
                .attachments
                .as_ref()
                .map(|atts| atts.iter().map(SendGridAttachment::from).collect()),
        };

        block_on(send_request(request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENV_API_KEY: &str = "SENDGRID_API_KEY";

    fn set_up() {
        let test_key = std::env::var(format!("TEST_{}", ENV_API_KEY))
            .expect("TEST_SENDGRID_API_KEY must be set");
        unsafe { std::env::set_var(ENV_API_KEY, test_key) };
    }

    #[test]
    fn test_validation_no_content() {
        let email = SimpleEmail {
            from_email: "test@example.com".to_string(),
            from_name: None,
            to_email: "recipient@example.com".to_string(),
            to_name: None,
            subject: "Test".to_string(),
            text_content: None,
            html_content: None,
        };

        let result = Component::send_simple(email);
        assert!(matches!(result, Err(SendError::ValidationError(_))));
    }

    #[test]
    fn test_validation_empty_personalizations() {
        let message = EmailMessage {
            sender: EmailAddress {
                email: "test@example.com".to_string(),
                name: None,
            },
            subject: "Test".to_string(),
            personalizations: vec![],
            content: vec![Content {
                mime_type: "text/plain".to_string(),
                value: "Hello".to_string(),
            }],
            reply_to: None,
            attachments: None,
        };

        let result = Component::send(message);
        assert!(matches!(result, Err(SendError::ValidationError(_))));
    }

    #[test]
    fn test_validation_empty_recipients() {
        let message = EmailMessage {
            sender: EmailAddress {
                email: "test@example.com".to_string(),
                name: None,
            },
            subject: "Test".to_string(),
            personalizations: vec![Personalization {
                to: vec![],
                cc: None,
                bcc: None,
                subject: None,
            }],
            content: vec![Content {
                mime_type: "text/plain".to_string(),
                value: "Hello".to_string(),
            }],
            reply_to: None,
            attachments: None,
        };

        let result = Component::send(message);
        assert!(matches!(result, Err(SendError::ValidationError(_))));
    }

    #[test]
    #[ignore] // Integration test - requires API key and valid email addresses
    fn test_send_simple_email() {
        set_up();

        let email = SimpleEmail {
            from_email: "sender@yourdomain.com".to_string(), // Must be verified in SendGrid
            from_name: Some("Test Sender".to_string()),
            to_email: "recipient@example.com".to_string(),
            to_name: Some("Test Recipient".to_string()),
            subject: "Test Email from Obelisk SendGrid Activity".to_string(),
            text_content: Some("This is a test email sent via the Obelisk SendGrid activity.".to_string()),
            html_content: Some("<h1>Test Email</h1><p>This is a test email sent via the <strong>Obelisk SendGrid activity</strong>.</p>".to_string()),
        };

        let result = Component::send_simple(email);
        assert!(result.is_ok(), "Failed to send email: {:?}", result.err());
        let response = result.unwrap();
        assert!(
            response.message_id.is_some(),
            "Expected message_id in response"
        );
    }
}
