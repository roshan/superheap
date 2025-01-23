use crate::types::Email;
use rusqlite::Connection;
use std::error::Error;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS emails (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                received_at DATETIME NOT NULL,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL, 
                subject TEXT NOT NULL,
                content TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn insert_email(&self, email: Email) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO emails (received_at, from_address, to_address, subject, content) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![email.recv_at(), email.from_address, email.to_address, email.subject, email.content],
        )?;
        Ok(())
    }

    pub fn get_emails(
        &self,
        num_entries_per_feed: u8,
        to_address: &String,
    ) -> Result<Vec<Email>, Box<dyn Error>> {
        let emails = self
            .conn
            .prepare(
                "SELECT id, received_at, from_address, to_address, subject, content
FROM (
   SELECT *,
   ROW_NUMBER() OVER (PARTITION BY subject ORDER BY received_at DESC) as rn
   FROM emails
   WHERE to_address = ?
)
WHERE rn = 1
ORDER BY id
LIMIT ?",
            )?
            .query_map(
                rusqlite::params![&to_address, &(num_entries_per_feed as i64)],
                |row| {
                    let recv : String = row.get(1)?;
                    Ok(Email {
                        id: row.get(0)?,
                        received_at: OffsetDateTime::parse(recv.as_str(), &Iso8601::DEFAULT).unwrap(),
                        from_address: row.get(2)?,
                        to_address: row.get(3)?,
                        subject: row.get(4)?,
                        content: row.get(5)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(emails)
    }
}
