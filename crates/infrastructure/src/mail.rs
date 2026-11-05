use application::{
    error::{AppError, AppResult},
    ports::ResetMailer,
};
use async_trait::async_trait;
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, client::Tls},
};

use crate::config;

const RESET_TEMPLATE: &str = include_str!("../resources/reset_template.html");

#[derive(Clone, Debug)]
struct SmtpSettings {
    host: String,
    username: String,
    password: String,
}

/// Sends password reset emails over SMTP.
pub struct SmtpResetMailer {
    company_name: String,
    company_domain: String,
    // Missing SMTP configuration is only reported when a mail is actually sent.
    smtp: Result<SmtpSettings, String>,
}

impl SmtpResetMailer {
    /// `company_name` and `company_domain` are required (they are also needed for the tokens);
    /// `SMTP_HOST`, `SMTP_USERNAME` and `SMTP_PASSWORD` are optional until a mail is sent.
    pub fn from_env() -> Self {
        let smtp = (|| {
            let var = |name: &str| {
                std::env::var(name)
                    .map_err(|_| format!("{name} environment variable not configured!"))
            };
            Ok(SmtpSettings {
                host: var("SMTP_HOST")?,
                username: var("SMTP_USERNAME")?,
                password: var("SMTP_PASSWORD")?,
            })
        })();
        Self {
            company_name: config::required("COMPANY_NAME"),
            company_domain: config::required("COMPANY_DOMAIN"),
            smtp,
        }
    }

    fn build_message(&self, email: &str, reset_key: &str) -> AppResult<Message> {
        let html = RESET_TEMPLATE
            .replace("{COMPANY_NAME}", &self.company_name)
            .replace(
                "{RESET_PASSWORD_URL}",
                &format!(
                    "{}/reset?key={reset_key}&email={email}",
                    self.company_domain
                ),
            );
        let internal = |what: &str| AppError::Internal(format!("Could not parse {what}!"));
        Message::builder()
            .from(
                format!("{} <noreply@{}>", self.company_name, self.company_domain)
                    .parse()
                    .map_err(|_| internal("sender address"))?,
            )
            .to(email.parse().map_err(|_| internal("recipient address"))?)
            .subject(format!(
                "Password Reset Requested for {}",
                self.company_name
            ))
            .header(ContentType::TEXT_HTML)
            .body(html)
            .map_err(|_| internal("email"))
    }
}

#[async_trait]
impl ResetMailer for SmtpResetMailer {
    async fn send_reset_link(&self, email: &str, reset_key: &str) -> AppResult<()> {
        let message = self.build_message(email, reset_key)?;
        let smtp = self.smtp.clone().map_err(AppError::Internal)?;
        let mailer = SmtpTransport::relay(&smtp.host)
            .map_err(|error| AppError::Internal(format!("Invalid SMTP host: {error}")))?
            .tls(Tls::None)
            .credentials(Credentials::new(smtp.username, smtp.password))
            .build();
        // The SMTP transport is blocking.
        tokio::task::spawn_blocking(move || mailer.send(&message))
            .await
            .map_err(|error| AppError::Internal(format!("Mail task failed: {error}")))?
            .map_err(|error| AppError::Internal(format!("Failed to send email: {error:?}")))?;
        println!("Reset email sent successfully to {email}");
        Ok(())
    }
}
