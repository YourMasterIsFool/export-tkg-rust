use lettre::{
    SmtpTransport, Transport,
    message::{Body, Mailbox, MessageBuilder, SinglePart, header::ContentType},
    transport::{smtp::authentication::Credentials, stub::Error},
};
use tera::Tera;

use crate::types::EmailTemplate;

#[derive(Clone)]
pub struct EmailWorker {
    tera: Tera,

    lettre_email: MessageBuilder,
    email_credential: Credentials,
}

impl EmailWorker {
    pub fn new(template_dir: &str) -> Self {
        let tera = Tera::new(&template_dir).expect("error template dir");

        let lettre_email = lettre::Message::builder().from(Mailbox::new(
            Some("no-reply@email.jobseeker.software".to_string()),
            "no-reply@email.jobseeker.software".parse().unwrap(),
        ));
        let email_credential = Credentials::new(
            "no-reply@email.jobseeker.software".to_owned(),
            "nLnGxdywwT3Rs6PY".to_owned(),
        );
        Self {
            tera: tera,
            lettre_email: lettre_email,
            email_credential: email_credential,
        }
    }

    pub fn send_email(&self, file: &str, email_template: &EmailTemplate) -> Result<String, tera::Error> {
        let context = tera::Context::from_serialize(&email_template)?;
        self.tera.render(&file, &context)
    }

    pub fn send_error_email(&self, email_template: &EmailTemplate) -> Result<(), ()> {
        let template = self.send_email("error.html", &email_template).unwrap();
        let email = &self
            .lettre_email
            .clone()
            .to(Mailbox::new(
                Some(email_template.client_email.clone()),
                email_template.client_email.clone().parse().unwrap(),
            ))
            .subject(email_template.subject.clone())
            .body(template)
            .unwrap();

        let mailer = SmtpTransport::relay("smtpdm-ap-southeast-1.aliyun.com")
            .unwrap()
            .credentials(self.email_credential.clone())
            .build();

        match mailer.send(&email) {
            Ok(_) => {
                println!("Email sent successfully!");
                Ok(())
            }
            Err(e) => panic!("Could not send email: {e:?}"),
        }
    }

    pub fn send_success(&self, email_tempate: &EmailTemplate) -> Result<(), tera::Error> {
        println!("send email ");
        let template = self.send_email("download.html", &email_tempate).unwrap();

        let email = &self
            .lettre_email
            .clone()
            .to(Mailbox::new(
                Some(email_tempate.client_email.clone()),
                email_tempate.client_email.clone().parse().unwrap(),
            ))
            .subject(email_tempate.subject.clone())
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(Body::new(template)),
            )
            .unwrap();

        let mailer = SmtpTransport::relay("smtpdm-ap-southeast-1.aliyun.com")
            .unwrap()
            .credentials(self.email_credential.clone())
            .build();

        match mailer.send(&email) {
            Ok(_) => {
                println!("Email sent successfully!");
                Ok(())
            }
            Err(e) => panic!("Could not send email: {e:?}"),
        }
    }
}
