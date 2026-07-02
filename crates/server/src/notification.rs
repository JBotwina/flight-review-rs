use serde::Serialize;

/// Upload notification configuration.
///
/// This starts as a config-gated no-op notifier: it captures the legacy uploader
/// email target and returns explicit metadata without sending SMTP email.
#[derive(Debug, Clone, Default)]
pub struct NotificationConfig {
    pub upload_notifications_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UploadNotificationResult {
    pub enabled: bool,
    pub recipient_present: bool,
    pub recipient: Option<String>,
    pub status: UploadNotificationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadNotificationStatus {
    Disabled,
    NoRecipient,
    Captured,
}

/// Normalize uploader email form input for response and notification metadata.
pub fn normalize_uploader_email(value: Option<String>) -> Option<String> {
    value
        .map(|email| email.trim().to_string())
        .filter(|email| !email.is_empty())
}

/// Prepare upload notification metadata without sending email.
pub fn prepare_upload_notification(
    config: &NotificationConfig,
    uploader_email: Option<&str>,
) -> UploadNotificationResult {
    let recipient = uploader_email.map(str::to_string);
    let recipient_present = recipient.is_some();
    let status = if !config.upload_notifications_enabled {
        UploadNotificationStatus::Disabled
    } else if recipient_present {
        UploadNotificationStatus::Captured
    } else {
        UploadNotificationStatus::NoRecipient
    };

    UploadNotificationResult {
        enabled: config.upload_notifications_enabled,
        recipient_present,
        recipient,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_blank_uploader_email_to_none() {
        assert_eq!(normalize_uploader_email(None), None);
        assert_eq!(normalize_uploader_email(Some("".to_string())), None);
        assert_eq!(normalize_uploader_email(Some("   ".to_string())), None);
    }

    #[test]
    fn trims_non_empty_uploader_email() {
        assert_eq!(
            normalize_uploader_email(Some("  pilot@example.com \n".to_string())),
            Some("pilot@example.com".to_string())
        );
    }

    #[test]
    fn disabled_notifier_captures_recipient_without_sending() {
        let result = prepare_upload_notification(
            &NotificationConfig {
                upload_notifications_enabled: false,
            },
            Some("pilot@example.com"),
        );

        assert!(!result.enabled);
        assert!(result.recipient_present);
        assert_eq!(result.recipient, Some("pilot@example.com".to_string()));
        assert_eq!(result.status, UploadNotificationStatus::Disabled);
    }

    #[test]
    fn enabled_notifier_reports_captured_recipient() {
        let result = prepare_upload_notification(
            &NotificationConfig {
                upload_notifications_enabled: true,
            },
            Some("pilot@example.com"),
        );

        assert!(result.enabled);
        assert!(result.recipient_present);
        assert_eq!(result.status, UploadNotificationStatus::Captured);
    }

    #[test]
    fn enabled_notifier_reports_missing_recipient() {
        let result = prepare_upload_notification(
            &NotificationConfig {
                upload_notifications_enabled: true,
            },
            None,
        );

        assert!(result.enabled);
        assert!(!result.recipient_present);
        assert_eq!(result.recipient, None);
        assert_eq!(result.status, UploadNotificationStatus::NoRecipient);
    }
}
