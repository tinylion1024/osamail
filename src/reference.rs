use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::{
    error::OsaMailError,
    model::{MailboxLocator, MessageLocator, REFERENCE_VERSION},
};

const MAX_REFERENCE_BYTES: usize = 8 * 1024;
pub const MAILBOX_REFERENCE_KIND: &str = "mailbox";

pub fn encode(locator: &MessageLocator) -> Result<String, OsaMailError> {
    validate(locator)?;
    let json = serde_json::to_vec(locator)?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

pub fn decode(reference: &str) -> Result<MessageLocator, OsaMailError> {
    let bytes = decode_bytes(reference)?;
    let locator: MessageLocator = serde_json::from_slice(&bytes)
        .map_err(|_| OsaMailError::InvalidReference("invalid locator JSON".to_owned()))?;
    validate(&locator)?;
    Ok(locator)
}

pub fn encode_mailbox(locator: &MailboxLocator) -> Result<String, OsaMailError> {
    validate_mailbox(locator)?;
    let json = serde_json::to_vec(locator)?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

pub fn decode_mailbox(reference: &str) -> Result<MailboxLocator, OsaMailError> {
    let bytes = decode_bytes(reference)?;
    let locator: MailboxLocator = serde_json::from_slice(&bytes)
        .map_err(|_| OsaMailError::InvalidReference("invalid mailbox locator JSON".to_owned()))?;
    validate_mailbox(&locator)?;
    Ok(locator)
}

fn decode_bytes(reference: &str) -> Result<Vec<u8>, OsaMailError> {
    if reference.is_empty() || reference.len() > MAX_REFERENCE_BYTES {
        return Err(OsaMailError::InvalidReference(
            "reference length is invalid".to_owned(),
        ));
    }
    if reference.chars().any(char::is_whitespace) {
        return Err(OsaMailError::InvalidReference(
            "reference must be one shell-safe token".to_owned(),
        ));
    }

    let bytes = URL_SAFE_NO_PAD
        .decode(reference)
        .map_err(|_| OsaMailError::InvalidReference("invalid Base64 encoding".to_owned()))?;
    Ok(bytes)
}

fn validate(locator: &MessageLocator) -> Result<(), OsaMailError> {
    if locator.version != REFERENCE_VERSION {
        return Err(OsaMailError::InvalidReference(
            "unsupported reference version".to_owned(),
        ));
    }
    if locator.account.trim().is_empty() {
        return Err(OsaMailError::InvalidReference(
            "account is missing".to_owned(),
        ));
    }
    if locator.mailbox_path.is_empty()
        || locator
            .mailbox_path
            .iter()
            .any(|component| component.trim().is_empty())
    {
        return Err(OsaMailError::InvalidReference(
            "mailbox path is invalid".to_owned(),
        ));
    }
    if locator.message_id <= 0 {
        return Err(OsaMailError::InvalidReference(
            "message id is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_mailbox(locator: &MailboxLocator) -> Result<(), OsaMailError> {
    if locator.kind != MAILBOX_REFERENCE_KIND {
        return Err(OsaMailError::InvalidReference(
            "reference is not a mailbox reference".to_owned(),
        ));
    }
    if locator.version != REFERENCE_VERSION {
        return Err(OsaMailError::InvalidReference(
            "unsupported reference version".to_owned(),
        ));
    }
    if locator.account.trim().is_empty() {
        return Err(OsaMailError::InvalidReference(
            "account is missing".to_owned(),
        ));
    }
    if locator.mailbox_path.is_empty()
        || locator
            .mailbox_path
            .iter()
            .any(|component| component.trim().is_empty())
    {
        return Err(OsaMailError::InvalidReference(
            "mailbox path is invalid".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locator() -> MessageLocator {
        MessageLocator {
            version: REFERENCE_VERSION,
            account: "个人账户 🚀".to_owned(),
            mailbox_path: vec!["收件箱".to_owned()],
            message_id: 42,
            internet_message_id: Some("<test@example.com>".to_owned()),
        }
    }

    fn mailbox_locator() -> MailboxLocator {
        MailboxLocator {
            kind: MAILBOX_REFERENCE_KIND.to_owned(),
            version: REFERENCE_VERSION,
            account: "个人账户 🚀".to_owned(),
            mailbox_path: vec!["项目".to_owned(), "归档".to_owned()],
        }
    }

    #[test]
    fn round_trips_unicode_locator() {
        let encoded = encode(&locator()).unwrap();
        assert!(!encoded.contains([' ', '=', '\n']));
        assert_eq!(decode(&encoded).unwrap(), locator());
    }

    #[test]
    fn rejects_damaged_reference() {
        let error = decode("not+url/safe").unwrap_err();
        assert_eq!(error.code(), "INVALID_REFERENCE");
    }

    #[test]
    fn rejects_semantically_invalid_locator() {
        let invalid = MessageLocator {
            message_id: 0,
            ..locator()
        };
        assert!(encode(&invalid).is_err());
    }

    #[test]
    fn round_trips_mailbox_reference() {
        let encoded = encode_mailbox(&mailbox_locator()).unwrap();
        assert!(!encoded.contains([' ', '=', '\n']));
        assert_eq!(decode_mailbox(&encoded).unwrap(), mailbox_locator());
    }

    #[test]
    fn rejects_message_reference_as_mailbox_reference() {
        let encoded = encode(&locator()).unwrap();
        assert!(decode_mailbox(&encoded).is_err());
    }
}
