use serde::{Deserialize, Serialize};

pub const REFERENCE_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub email_addresses: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageLocator {
    pub version: u8,
    pub account: String,
    pub mailbox_path: Vec<String>,
    pub message_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internet_message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSummary {
    #[serde(rename = "ref")]
    pub reference: String,
    pub account: String,
    pub mailbox: String,
    pub sender: String,
    pub subject: String,
    pub received_at: Option<String>,
    pub unread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageDetail {
    #[serde(rename = "ref")]
    pub reference: String,
    pub account: String,
    pub mailbox: String,
    pub sender: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub received_at: Option<String>,
    pub unread: bool,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub ready: bool,
    pub architecture: String,
    pub mail_version: Option<String>,
    pub account_count: usize,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendRequest {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub account: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendResult {
    pub sent: bool,
    pub dry_run: bool,
    pub account: Option<String>,
    pub recipient_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum AutomationRequest {
    Doctor,
    Accounts,
    ListMessages(ListMessagesRequest),
    ShowMessage(ShowMessageRequest),
    OpenMessage(OpenMessageRequest),
    SendMessage(SendRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<AutomationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListMode {
    Recent,
    Unread,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListMessagesRequest {
    pub mode: ListMode,
    pub limit: u16,
    pub account: Option<String>,
    pub mailbox: Option<String>,
    pub count_only: bool,
    pub query: Option<String>,
    pub unread: bool,
    pub from: Option<String>,
    pub subject: Option<String>,
    pub search_body: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowMessageRequest {
    pub locator: MessageLocator,
    pub include_headers: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenMessageRequest {
    pub locator: MessageLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawMessageSummary {
    pub locator: MessageLocator,
    pub sender: String,
    pub subject: String,
    pub received_at: Option<String>,
    pub unread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawMessageDetail {
    pub locator: MessageLocator,
    pub sender: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub received_at: Option<String>,
    pub unread: bool,
    pub body: String,
    pub headers: Option<String>,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorAutomationData {
    pub mail_version: String,
    pub account_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountsData {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessagesData {
    pub messages: Vec<RawMessageSummary>,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenResult {
    pub opened: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Success<T> {
    pub ok: bool,
    pub data: T,
}

impl<T> Success<T> {
    pub const fn new(data: T) -> Self {
        Self { ok: true, data }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountsOutput {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessagesOutput {
    pub messages: Vec<MessageSummary>,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountOutput {
    pub count: usize,
}
