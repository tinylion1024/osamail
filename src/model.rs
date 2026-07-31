use serde::{Deserialize, Serialize};

pub const REFERENCE_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub email_addresses: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxLocator {
    pub kind: String,
    pub version: u8,
    pub account: String,
    pub mailbox_path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxSummary {
    #[serde(rename = "ref")]
    pub reference: String,
    pub account: String,
    pub path: Vec<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkAction {
    Read,
    Unread,
    Flag,
    Unflag,
}

impl MarkAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Unread => "unread",
            Self::Flag => "flagged",
            Self::Unflag => "unflagged",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkMessageRequest {
    pub locator: MessageLocator,
    pub action: MarkAction,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkAutomationData {
    pub already_set: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkOutcome {
    Changed,
    AlreadySet,
    WouldChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkResult {
    #[serde(rename = "ref")]
    pub reference: String,
    pub action: MarkAction,
    pub outcome: MarkOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkBatchItem {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<MarkOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchItemError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkBatchResult {
    pub action: MarkAction,
    pub dry_run: bool,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub items: Vec<MarkBatchItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationAction {
    Move,
    Archive,
}

impl OrganizationAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Archive => "archive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveMessageRequest {
    pub locator: MessageLocator,
    pub destination: MailboxLocator,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveAutomationData {
    pub already_there: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationOutcome {
    Moved,
    AlreadyThere,
    WouldMove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationItem {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<OrganizationOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchItemError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationResult {
    pub action: OrganizationAction,
    #[serde(rename = "destination_ref")]
    pub destination_reference: String,
    pub dry_run: bool,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub items: Vec<OrganizationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum AutomationRequest {
    Doctor,
    Accounts,
    ListMailboxes(ListMailboxesRequest),
    ListMessages(ListMessagesRequest),
    ShowMessage(ShowMessageRequest),
    OpenMessage(OpenMessageRequest),
    MarkMessage(MarkMessageRequest),
    MoveMessage(MoveMessageRequest),
    SendMessage(SendRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListMailboxesRequest {
    pub account: Option<String>,
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
    pub titles_only: bool,
    pub account: Option<String>,
    pub mailbox: Option<String>,
    pub count_only: bool,
    pub query: Option<String>,
    pub unread: bool,
    pub from: Option<String>,
    pub subject: Option<String>,
    pub search_body: bool,
    pub since: Option<String>,
    pub before: Option<String>,
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
pub struct MailboxesData {
    pub mailboxes: Vec<RawMailboxSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawMailboxSummary {
    pub account: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessagesData {
    #[serde(default)]
    pub messages: Vec<RawMessageSummary>,
    #[serde(default)]
    pub titles: Vec<String>,
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
pub struct MailboxesOutput {
    pub mailboxes: Vec<MailboxSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessagesOutput {
    pub messages: Vec<MessageSummary>,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitlesOutput {
    pub titles: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountOutput {
    pub count: usize,
}
