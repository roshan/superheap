use rss::{Channel, ChannelBuilder, ItemBuilder};
use std::error::Error;
use std::fs;
use std::io::Write;
use superheap::db::Database;
use superheap::types::{Config, FeedConfig};


pub fn generate_feeds(cfg: Config) -> Result<(), Box<dyn Error>> {
    let database = Database::new(&cfg.db_path)?;
    std::fs::create_dir_all(&cfg.feed_path)?;
    for feed_cfg in cfg.dst_email_to_feed.iter() {
        let feed = generate_feed(feed_cfg, &database, feed_cfg.feed_name.clone(), cfg.num_entries_per_feed);
        match feed {
            Ok(channel) => {
                let feed_path = format!("{}/{}.xml", cfg.feed_path, feed_cfg.feed_name);
                let mut file = fs::File::create(feed_path.clone())?;
                file.write_all(channel.to_string().as_bytes())?;
                println!("Wrote feed {} to {}", feed_cfg.feed_name, feed_path);
            },
            Err(e) => {
                eprintln!("Failed to generate feed: {}", e);
            }
        }
    }

    Ok(())
}

pub fn generate_feed(feed_cfg: &FeedConfig, database: &Database, feed_name: String, num_entries_per_feed: u8) -> Result<Channel, Box<dyn Error>> {
    let mut items = Vec::new();
    let emails = database.get_emails(num_entries_per_feed, &feed_cfg.to_email)?;

    for email in emails {
        let recv_at = email.recv_at();
        items.push(ItemBuilder::default()
            .title(email.subject)
            .description(email.content.clone())
            .content(email.content.clone())
            .author(feed_cfg.feed_author.clone())
            .pub_date(recv_at)
            .build());
        eprintln!("Added email to feed: {}", email.content.clone());
    }
    println!("Generated feed for {}", feed_name);
    println!("{} items in feed", items.len());

    let channel = ChannelBuilder::default()
        .title(feed_name.clone())
        .link("https://example.com")
        .description(format!("Email feed for {}", feed_name))
        .items(items)
        .build();

    Ok(channel)
}

