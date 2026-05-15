use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub struct SmtpProvider {
    host: String,
    port: u16,
    user: String,
    pass: String,
    from: String,
}

impl SmtpProvider {
    pub fn new(host: &str, port: u16, user: &str, pass: &str, from: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            user: user.to_string(),
            pass: pass.to_string(),
            from: from.to_string(),
        }
    }

    fn build_transport(&self) -> SmtpTransport {
        if self.port == 25 {
            SmtpTransport::builder_dangerous(&self.host)
                .port(self.port)
                .build()
        } else if !self.user.is_empty() && !self.pass.is_empty() {
            let creds = Credentials::new(self.user.clone(), self.pass.clone());
            SmtpTransport::relay(&self.host)
                .unwrap_or_else(|e| {
                    log::error!("SMTP relay setup failed: {e}");
                    SmtpTransport::builder_dangerous(&self.host)
                })
                .credentials(creds)
                .port(self.port)
                .build()
        } else {
            SmtpTransport::relay(&self.host)
                .unwrap_or_else(|e| {
                    log::error!("SMTP relay setup failed: {e}");
                    SmtpTransport::builder_dangerous(&self.host)
                })
                .port(self.port)
                .build()
        }
    }

    pub fn send_email(&self, to: &[String], subject: &str, body_html: &str) -> Result<(), String> {
        let from_mailbox: Mailbox = self
            .from
            .parse()
            .map_err(|e| format!("Invalid from address: {e}"))?;

        let to_mailboxes: Vec<Mailbox> = to
            .iter()
            .filter_map(|addr| addr.parse::<Mailbox>().ok())
            .collect();

        if to_mailboxes.is_empty() {
            return Err("No valid recipient addresses".into());
        }

        let mut builder = Message::builder()
            .from(from_mailbox)
            .subject(subject.to_string());

        for addr in &to_mailboxes {
            builder = builder.to(addr.clone());
        }

        let email = builder
            .multipart(
                lettre::message::MultiPart::alternative().singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(body_html.to_string()),
                ),
            )
            .map_err(|e| format!("Failed to build email: {e}"))?;

        let transport = self.build_transport();

        transport
            .send(&email)
            .map_err(|e| format!("SMTP send failed: {e}"))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn health_check(&self) -> bool {
        self.build_transport().test_connection().unwrap_or(false)
    }
}
