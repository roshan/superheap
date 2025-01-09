use mail_parser::MessageParser;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug)]
pub struct Email {
    pub id: u64,
    pub received_at: OffsetDateTime,
    pub from_address: String,
    pub to_address: String,
    pub subject: String,
    pub content: String,
}

impl Email {
    pub fn recv_at(&self) -> String {
        // Format as RFC3339/ISO8601 which SQLite's datetime functions can parse
        self.received_at.format(&Rfc3339).unwrap()
    }
}

impl Email {
    pub fn parse(raw_data: &[u8], mail_id: u64) -> Option<Email> {
        // Use mail_parser to handle MIME structure
        let message = MessageParser::new().parse(raw_data)?;

        // Extract headers
        let from = message.from()?.first()?.address()?.to_string();
        let to = message.to()?.first()?.address()?.to_string();
        let subject = message.subject()?.to_string();

        // Extract body content
        let content = message.body_html(0).unwrap_or_default().to_string();

        Some(Email {
            id: mail_id,
            received_at: OffsetDateTime::now_utc(),
            from_address: from,
            to_address: to,
            subject,
            content,
        })
    }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedConfig {
    pub display_name: String,
    pub to_email: String,
    pub feed_name: String,
    pub feed_author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bind_ip: String,
    pub port: u16,
    pub dst_email_to_feed: Vec<FeedConfig>,
    pub db_path: String,
    pub num_entries_per_feed: u8,
    pub feed_path: String,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        port: u16,
        to_email: String,
        display_name: String,
        feed_name: String,
        feed_author: String,
        db_path: String,
        num_entries_per_feed: u8,
        feed_path: String,
    ) -> Config {
        let dst_email_to_feed = vec![FeedConfig {
            display_name,
            to_email,
            feed_name,
            feed_author,
        }];

        Config {
            bind_ip: "0.0.0.0".to_string(),
            port,
            dst_email_to_feed,
            db_path,
            num_entries_per_feed,
            feed_path,
        }
    }

}
