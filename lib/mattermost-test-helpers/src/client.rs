use anyhow::Result;

/// HTTP client authenticated with a Mattermost token
pub struct AuthenticatedClient {
    pub base_url: String,
    pub client: reqwest::Client,
    pub token: String,
    pub user_id: String,
}

impl AuthenticatedClient {
    /// Post a message to a channel
    pub async fn post_message(&self, channel_id: &str, message: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "channel_id": channel_id,
            "message": message,
        });

        let response = self
            .client
            .post(format!("{}/api/v4/posts", self.base_url))
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to post message: {} - {}", status, body);
        }

        Ok(response.json().await?)
    }

    /// Add a reaction to a post
    pub async fn add_reaction(&self, post_id: &str, emoji_name: &str) -> Result<()> {
        let create_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let payload = serde_json::json!({
            "user_id": self.user_id,
            "post_id": post_id,
            "emoji_name": emoji_name,
            "create_at": create_at,
        });

        let response = self
            .client
            .post(format!("{}/api/v4/reactions", self.base_url))
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to add reaction: {} - {}", status, body);
        }

        Ok(())
    }
}
