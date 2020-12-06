use reqwest::{Client, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct TelegramResponse {
    ok: bool,
}

pub struct TelegramClient {
    url: String,
}

impl TelegramClient {
    pub fn new(token: String) -> Self {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        TelegramClient { url }
    }

    pub async fn notify(&self, chat_id: String, message: &str) -> Result<()> {
        let client = Client::new();
        client
            .get(&self.url)
            .query(&[
                ("chat_id", &chat_id[..]),
                ("parse_mode", "Markdown"),
                ("text", message),
            ])
            .send()
            .await?
            .json::<TelegramResponse>()
            .await?;
        Ok(())
    }
}
