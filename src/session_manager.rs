use std::{net::TcpStream, process::exit};

use imap::{types::Fetch, Client, Session};
use native_tls::{TlsConnector, TlsStream};

use crate::config::Config;

pub struct SessionManager {
    pub src: Session<TlsStream<TcpStream>>,
    include: Vec<String>,
    exclude: Vec<String>,
    pub dst: Session<TlsStream<TcpStream>>,
}

impl SessionManager {
    pub fn new(config: Config) -> Self {
    
        tracing::debug!("⏳ Connecting to Source server...");
        let src_session = SessionManager::connect(&config.src.host, &config.src.user, &config.src.password);
        tracing::debug!("✅ Connected to Source {} server", config.src.host);

        tracing::debug!("⏳ Connecting to Destination server...");
        let dst_session = SessionManager::connect(&config.dst.host, &config.dst.user, &config.dst.password);
        tracing::debug!("✅ Connected to Destination {} server", config.dst.host);

        let include = config.src.include
            .unwrap_or_else(|| "*".to_owned())
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect();
        tracing::debug!("Including {:?} mailboxes", include);

        let exclude = config.src.exclude
            .unwrap_or_else(|| "".to_owned())
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect();
        tracing::debug!("Excluding {:?} mailboxes", exclude);
        
        Self{
            src: src_session,
            include,
            exclude,
            dst: dst_session,
        }
    }

    pub fn list_all(&mut self) -> Vec<String> {
        let list_src = match self.src.list(Some("*"), Some("*")) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("🚨 {}", e);
                exit(1)
            }
        };
        list_src
            .iter()
            .map(|i| i.name().to_string())
            .collect()
    }

    pub fn sync(&mut self) {

        let list_included = self.list();
        tracing::info!("Syncing {:?} mailboxes", list_included);

        for mailbox in list_included.iter() {
            let mbox = self.src.select(mailbox).unwrap();
            if mbox.exists > 0 {
                tracing::info!("⏳ [{mailbox}] Syncing {} emails", mbox.exists);

                // FETCH from SRC
                let seq = match mbox.exists {
                    1 => "1".to_string(),
                    _ => format!("1:{}", mbox.exists)
                };
                let fetch = self.src
                    .fetch(seq, "(ENVELOPE BODY[])")
                    .expect("could not fetch");

                let mut append_ok = 0;
                fetch
                    .iter()
                    .for_each(|msg| {
                    // Extract Message-ID
                    let msg_id = msg
                        .envelope()
                        .expect("Message did not have an Envelope")
                        .message_id
                        .expect("Envelope did not have a Message-ID");
                    let msg_id = std::str::from_utf8(msg_id)
                        .expect("Message-ID was not valid utf-8")
                        .to_string();
                    
                    // Search Message-ID in DST
                    match self.search_msg(mailbox, &msg_id) {
                        Ok(0) => {
                            // Append message if it does not exist
                            match self.append_msg(&mailbox, &msg_id, msg) {
                                Ok(_) => append_ok += 1,
                                Err(e) => tracing::error!("🚨 [{mailbox}] APPEND Error {}: {}", msg_id, e),
                            }
                        },
                        Ok(uid) => tracing::debug!("[{mailbox}] SEARCH: Message {} already exists with UID {}", msg_id, uid),
                        Err(e) => tracing::debug!("🚨 [{mailbox}] SEARCH Message {}: {}", msg_id, e),
                    }
                });
                tracing::info!("✅ [{mailbox}] Synced {}/{} emails", append_ok, mbox.exists);
            }
        }
    }

    pub fn logout(&mut self) -> anyhow::Result<()> {
        self.src.logout()?;
        self.dst.logout()?;
        Ok(())
    }

    fn search_msg(&mut self, mailbox: &str, msg_id: &str) -> anyhow::Result<u32> {
        let _ = self.dst.select(mailbox).unwrap();
        match self.dst.search(format!("HEADER Message-ID {}", msg_id)) {
            Ok(response) => {
                if response.is_empty() {
                    // Message does not exist on DST
                    Ok(0)
                } else {
                    // Message already exist on DST
                    let uid = response.iter().take(1).next().unwrap();
                    Ok(*uid)
                }
            },
            Err(e) => {
                tracing::debug!("🚨 [{mailbox}] SEARCH Message {}: {}", msg_id, e);
                Err(e.into())
            },
        }
    }

    fn append_msg(&mut self, mailbox: &str, msg_id: &str, msg: &Fetch) -> anyhow::Result<()> {
        let body = msg.body().expect(&format!("Could not extract body from msg {}", msg_id));
        match self.dst.append_with_flags(&mailbox, body, msg.flags()) {
            Ok(_) => {
                tracing::info!("[{mailbox}] APPEND {}", msg_id);
                Ok(())
            },
            Err(e) => Err(e.into())
        }
    }

    fn list(&mut self) -> Vec<String> {
        let include = self.include.join(" ");
        let list_src = match self.src.list(Some(&include), Some("*")) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("🚨 {}", e);
                exit(1)
            }
        };
        list_src
            .iter()
            .map(|i| i.name().to_string())
            .filter(|i| !self.exclude.contains(i))
            .collect()
    }

    fn connect(host: &str, user: &str, password: &str) -> Session<TlsStream<TcpStream>> {
        let connector = match TlsConnector::new() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("🚨 [TLS] {}", e);
                exit(1);
            }
        };

        let domain = format!("{}:{}", host, 993);

        let tcp = match TcpStream::connect(&domain) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("🚨 [TCP] {}", e);
                drop(connector);
                exit(1);
            }
        };

        let tls = match connector.connect(&host, tcp) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("🚨 [TLS Connect] {}", e);
                drop(connector);
                exit(1);
            }
        };

        let client = Client::new(tls);
        match client.login(user, password) {
            Ok(s) => s,
            Err((e, _)) => {
                tracing::error!("🚨 [IMAP] {}", e);
                drop(connector);
                exit(1);
            }
        }
    }
}
