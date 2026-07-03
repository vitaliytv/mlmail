pub struct Endpoints {
    pub google_token: String,
    pub gmail_label_inbox: String,
    pub gmail_labels: String,
    pub gmail_messages_list: String,
    pub gmail_batch_modify: String,
    pub gmail_filters: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            google_token: crate::auth::token_exchange::GOOGLE_TOKEN_URL.to_string(),
            gmail_label_inbox: crate::gmail::GMAIL_LABEL_INBOX_URL.to_string(),
            gmail_labels: crate::gmail::GMAIL_LABELS_URL.to_string(),
            gmail_messages_list: crate::gmail::GMAIL_MESSAGES_LIST_URL.to_string(),
            gmail_batch_modify: crate::gmail::GMAIL_BATCH_MODIFY_URL.to_string(),
            gmail_filters: crate::gmail::GMAIL_FILTERS_URL.to_string(),
        }
    }
}
