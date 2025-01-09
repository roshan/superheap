use anyhow::Context;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use superheap::db;
use superheap::types::Config;
use superheap::types::Email;

static MAIL_ID: AtomicU64 = AtomicU64::new(1);

fn handle_smtp_client(stream: &mut TcpStream, tx: Sender<Email>) -> anyhow::Result<()> {
    // Send greeting
    let response = "220 smtp.example.com Simple Mail Transfer Service Ready\r\n";
    stream
        .write(response.as_bytes())
        .context("Failed to send greeting")?;

    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => return Ok(()), // Connection closed
            Ok(n) => {
                let received = String::from_utf8_lossy(&buffer[..n]);
                let command = received.trim().to_uppercase();

                match command.as_str() {
                    cmd if cmd.starts_with("HELO") || cmd.starts_with("EHLO") => {
                        stream.write_all(b"250 Hello\r\n")?;
                    }
                    cmd if cmd.starts_with("MAIL FROM:") => {
                        stream.write_all(b"250 Ok\r\n")?;
                    }
                    cmd if cmd.starts_with("RCPT TO:") => {
                        stream.write_all(b"250 Ok\r\n")?;
                    }
                    "DATA" => {
                        return handle_data(&mut *stream, tx);
                    }
                    "QUIT" => {
                        stream
                            .write(b"221 Bye\r\n")
                            .context("Couldn't respond to quit")?;
                        return Ok(());
                    }
                    c => {
                        eprintln!("Unknown command: {}", c);
                        stream
                            .write(b"500 Unknown command\r\n")
                            .context(format!("Unknown command: {}", c))?;
                        return Err(anyhow::anyhow!("Unknown command {}", c));
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                return Err(e.into());
            }
        }
    };
}

fn handle_data(stream: &mut TcpStream, tx: Sender<Email>) -> anyhow::Result<()> {
    stream
        .write(b"354 Start mail input; end with <CRLF>.<CRLF>\r\n")
        .context("Mail not processed")?;
    let mail_id = MAIL_ID.fetch_add(1, Ordering::Relaxed);
    let mut data = Vec::new();

    {
        let mut reader = BufReader::new(&mut *stream);
        // Read until we see the end marker
        loop {
            let mut line = Vec::new();
            let n = reader.read_until(b'\n', &mut line)?;
            if n == 0 || line == b".\r\n" || data.len() >= 10_000_000 {
                break;
            }
            data.extend_from_slice(&line);
        }
    }

    if let Some(email) = Email::parse(&data, mail_id) {
        tx.send(email).context("Failed to queue email")?;
    }

    stream
        .write(format!("250 Ok: queued as {}\r\n", mail_id).as_bytes())
        .context("Failed to inform about queue")?;
    stream.write(b"221 Bye\r\n").context("Failed to send bye")?;
    Ok(())
}

fn process_messages(rx: mpsc::Receiver<Email>, handler: Box<dyn EmailHandler>) {
    while let Ok(email) = rx.recv() {
        let email_id = email.id;
        let from = email.from_address.clone();
        let to = email.to_address.clone();
        match handler.handle(email) {
            Ok(_) => {
                eprintln!(
                    "Successfully handled email {} from {} to {}",
                    email_id, from, to
                );
            }
            Err(e) => {
                eprintln!("Failed to process email: {}", e);
            }
        }
    }
}

pub trait EmailHandler: Send {
    fn handle(&self, email: Email) -> Result<(), Box<dyn std::error::Error>>;
}

struct DebugHandler;
impl EmailHandler for DebugHandler {
    fn handle(&self, email: Email) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received email: {:?}", email);
        Ok(())
    }
}

impl EmailHandler for db::Database {
    fn handle(&self, email: Email) -> Result<(), Box<dyn std::error::Error>> {
        self.insert_email(email).map_err(|e| e.into())
    }
}

pub fn start_mail_server(config: Config, is_debug: bool) {
    let listener = TcpListener::bind(format!("{}:{}", config.bind_ip, config.port))
        .expect("Failed to bind to address");
    println!("Mail server listening on port {}", config.port);
    let (tx, rx) = mpsc::channel::<Email>();

    let handler: Box<dyn EmailHandler> = if is_debug {
        Box::new(DebugHandler)
    } else {
        Box::new(db::Database::new(&config.db_path).expect("Failed to initialize database"))
    };

    thread::spawn(move || {
        process_messages(rx, handler);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let tx = tx.clone();
                thread::spawn(move || {
                    stream
                        .peer_addr()
                        .map(|addr| println!("Connection from: {}", addr))
                        .ok();
                    handle_smtp_client(&mut stream, tx)
                        .unwrap_or_else(|e| eprintln!("Failed to handle client: {}", e));
                    stream.flush().ok();
                    stream.shutdown(std::net::Shutdown::Both).ok();
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
