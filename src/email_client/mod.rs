use crate::domain::SubscriberEmail;
use mailslurp::{
    apis::{
        configuration,
        inbox_controller_api::{self, SendEmailAndConfirmParams},
    },
    models::SendEmailOptions,
};
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
    configuration: mailslurp::apis::configuration::Configuration,
    inbox_id: String,
    _sender: SubscriberEmail,
}

impl EmailClient {
    pub async fn new(authorization_token: Secret<String>) -> Self {
        const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(60_000);
        let client = reqwest::ClientBuilder::new()
            .timeout(TIMEOUT)
            .connect_timeout(TIMEOUT)
            .build()
            .unwrap();

        // configure mailslurp with base path, api key, and reqwest client
        let configuration = configuration::Configuration {
            // required fields
            base_path: "https://api.mailslurp.com".to_owned(),
            api_key: Some(configuration::ApiKey {
                prefix: None,
                key: authorization_token.expose_secret().to_owned(),
            }),
            client,
            // leave as none
            user_agent: None,
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
        };

        let mut create_inbox_dto = mailslurp::models::CreateInboxDto::new();
        create_inbox_dto.name = Some("test inbox".into());
        let inbox_params = inbox_controller_api::CreateInboxWithOptionsParams { create_inbox_dto };
        // methods are async and return results
        let inbox =
            match inbox_controller_api::create_inbox_with_options(&configuration, inbox_params)
                .await
            {
                Ok(inbox) => inbox,
                Err(e) => panic!("{}", e.to_string()),
            };
        Self {
            configuration,
            inbox_id: inbox.id.unwrap(),
            _sender: SubscriberEmail::parse(inbox.email_address.unwrap()).unwrap(),
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        _text_content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let send_email_params = SendEmailAndConfirmParams {
            inbox_id: self.inbox_id.to_owned(),
            send_email_options: Some(SendEmailOptions {
                // common params
                to: Some(vec![recipient.to_string()]),
                body: Some(html_content.to_owned()),
                html: Some(true),
                use_inbox_name: Some(true),
                add_tracking_pixel: Some(false),
                subject: Some(subject.to_owned()),
                is_html: Some(true),
                // extras
                attachments: None,
                bcc: None,
                cc: None,
                charset: None,
                from: None,
                reply_to: None,
                send_strategy: None,
                template: None,
                template_variables: None,
                to_contacts: None,
                to_group: None,
            }),
        };
        inbox_controller_api::send_email_and_confirm(&self.configuration, send_email_params)
            .await?;
        Ok(())
    }
}
