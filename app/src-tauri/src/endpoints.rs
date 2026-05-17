pub struct Endpoints {
    pub google_token: String,
    pub gmail_label_inbox: String,
    pub gmail_messages_list: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            google_token: crate::auth::token_exchange::GOOGLE_TOKEN_URL.to_string(),
            gmail_label_inbox: crate::gmail::GMAIL_LABEL_INBOX_URL.to_string(),
            gmail_messages_list: crate::gmail::GMAIL_MESSAGES_LIST_URL.to_string(),
        }
    }
}
