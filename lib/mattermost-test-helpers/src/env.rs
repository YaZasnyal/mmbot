use crate::client::AuthenticatedClient;
use anyhow::{Context, Result};
use mattermost_api::apis::configuration::Configuration;
use std::env;
use std::sync::Arc;

/// Test environment for Mattermost integration tests
///
/// Expects Mattermost to be running externally (via `docker-compose up -d` from project root).
/// Set `MATTERMOST_URL` environment variable to override default (`http://localhost:8065`).
///
/// Creates all necessary resources (admin user, team, channel, bot) automatically.
pub struct MattermostTestEnv {
    pub base_url: String,
    pub admin_token: String,
    pub admin_user_id: String,
    pub team_id: String,
    pub channel_id: String,
    bot_token: String,
    bot_user_id: String,
}

impl MattermostTestEnv {
    /// Create a fully initialized test environment
    ///
    /// This will:
    /// - Wait for Mattermost to be ready
    /// - Create a unique admin user
    /// - Create a team
    /// - Create a channel
    /// - Create a bot account
    ///
    /// Returns ready-to-use environment with all resources created.
    pub async fn new() -> Result<Self> {
        let base_url =
            env::var("MATTERMOST_URL").unwrap_or_else(|_| "http://localhost:8065".to_string());

        tracing::info!("Initializing test environment...");
        tracing::info!("Mattermost URL: {}", base_url);

        // Wait for Mattermost to be ready
        tracing::info!("Waiting for Mattermost to be ready...");
        Self::wait_for_ready(&base_url).await?;
        tracing::info!("Mattermost is ready");

        // Generate unique test ID
        let test_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        tracing::info!("Test ID: {}", test_id);

        // Get or create admin user (use fixed credentials for all tests)
        let admin_username = env::var("MM_ADMIN_USER").unwrap_or_else(|_| "testadmin".to_string());
        let admin_password =
            env::var("MM_ADMIN_PASS").unwrap_or_else(|_| "TestAdmin123!".to_string());
        tracing::info!("Using admin user: {}", admin_username);
        let (admin_token, admin_user_id) =
            Self::get_or_create_admin(&base_url, &admin_username, &admin_password).await?;
        tracing::info!("Admin user ready");

        // Create team
        let team_name = format!("team{}", test_id);
        tracing::info!("Creating team: {}", team_name);
        let team_id = Self::create_team(&base_url, &admin_token, &team_name).await?;
        tracing::info!("Team created: {}", team_id);

        // Create channel
        let channel_name = format!("channel{}", test_id);
        tracing::info!("Creating channel: {}", channel_name);
        let channel_id =
            Self::create_channel(&base_url, &admin_token, &team_id, &channel_name).await?;
        tracing::info!("Channel created: {}", channel_id);

        // Create bot
        let bot_name = format!("bot{}", test_id);
        tracing::info!("Creating bot: {}", bot_name);
        let (bot_token, bot_user_id) =
            Self::create_bot(&base_url, &admin_token, &team_id, &bot_name).await?;
        tracing::info!("Bot created and configured");

        // Add bot to channel
        tracing::info!("Adding bot to channel...");
        Self::add_user_to_channel(&base_url, &admin_token, &channel_id, &bot_user_id).await?;
        tracing::info!("Bot added to channel");

        tracing::info!("Test environment ready!");

        Ok(Self {
            base_url,
            admin_token,
            admin_user_id,
            team_id,
            channel_id,
            bot_token,
            bot_user_id,
        })
    }

    /// Get bot configuration ready for Bot::with_config()
    pub fn bot_config(&self) -> Arc<Configuration> {
        Arc::new(Configuration {
            base_path: self.base_url.clone(),
            bearer_access_token: Some(self.bot_token.clone()),
            ..Configuration::default()
        })
    }

    /// Get bot's access token
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    /// Get bot's user ID
    pub fn bot_user_id(&self) -> &str {
        &self.bot_user_id
    }

    /// Get authenticated HTTP client for making API calls
    ///
    /// By default returns client authenticated as admin.
    /// To create client for other users, pass their token and user_id.
    pub fn http_client(&self, token: Option<&str>, user_id: Option<&str>) -> AuthenticatedClient {
        AuthenticatedClient {
            base_url: self.base_url.clone(),
            client: reqwest::Client::new(),
            token: token.unwrap_or(&self.admin_token).to_string(),
            user_id: user_id.unwrap_or(&self.admin_user_id).to_string(),
        }
    }

    async fn wait_for_ready(base_url: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let ping_url = format!("{}/api/v4/system/ping", base_url);

        for attempt in 1..=30 {
            match client.get(&ping_url).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                _ => {
                    if attempt % 5 == 0 {
                        tracing::info!("Still waiting... attempt {}/30", attempt);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }

        anyhow::bail!("Mattermost is not responding after 30 attempts")
    }

    /// Get or create admin user (idempotent - safe to call multiple times)
    async fn get_or_create_admin(
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<(String, String)> {
        let client = reqwest::Client::new();

        // Try to login first (user might already exist)
        let login_payload = serde_json::json!({
            "login_id": username,
            "password": password,
        });

        let login_response = client
            .post(format!("{}/api/v4/users/login", base_url))
            .json(&login_payload)
            .send()
            .await
            .context("Failed to login")?;

        if login_response.status().is_success() {
            // User exists, login successful
            let token = login_response
                .headers()
                .get("Token")
                .context("No token in login response")?
                .to_str()
                .context("Invalid token format")?
                .to_string();

            let user: serde_json::Value = login_response.json().await?;
            let user_id = user["id"]
                .as_str()
                .context("No user_id in login response")?
                .to_string();

            return Ok((token, user_id));
        }

        // User doesn't exist, try to create (works only on fresh Mattermost with open signups)
        let user_payload = serde_json::json!({
            "email": format!("{}@example.com", username),
            "username": username,
            "password": password,
        });

        let create_response = client
            .post(format!("{}/api/v4/users", base_url))
            .json(&user_payload)
            .send()
            .await
            .context("Failed to create admin user")?;

        if !create_response.status().is_success() {
            let status = create_response.status();
            let body = create_response.text().await?;
            anyhow::bail!("Failed to create admin user: {} - {}. Hint: Set MM_ADMIN_USER and MM_ADMIN_PASS environment variables if Mattermost is already initialized.", status, body);
        }

        // Now login with the newly created user
        let login_response = client
            .post(format!("{}/api/v4/users/login", base_url))
            .json(&login_payload)
            .send()
            .await
            .context("Failed to login as newly created admin")?;

        let token = login_response
            .headers()
            .get("Token")
            .context("No token in login response")?
            .to_str()
            .context("Invalid token format")?
            .to_string();

        let user: serde_json::Value = login_response.json().await?;
        let user_id = user["id"]
            .as_str()
            .context("No user_id in login response")?
            .to_string();

        Ok((token, user_id))
    }

    async fn create_team(base_url: &str, admin_token: &str, name: &str) -> Result<String> {
        let client = reqwest::Client::new();

        let team_payload = serde_json::json!({
            "name": name,
            "display_name": format!("{} Team", name),
            "type": "O",
        });

        let response = client
            .post(format!("{}/api/v4/teams", base_url))
            .bearer_auth(admin_token)
            .json(&team_payload)
            .send()
            .await
            .context("Failed to create team")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to create team: {} - {}", status, body);
        }

        let team: serde_json::Value = response.json().await?;
        let team_id = team["id"]
            .as_str()
            .context("No id in team response")?
            .to_string();

        Ok(team_id)
    }

    async fn create_channel(
        base_url: &str,
        admin_token: &str,
        team_id: &str,
        name: &str,
    ) -> Result<String> {
        let client = reqwest::Client::new();

        let channel_payload = serde_json::json!({
            "team_id": team_id,
            "name": name,
            "display_name": format!("{} Channel", name),
            "type": "O",
        });

        let response = client
            .post(format!("{}/api/v4/channels", base_url))
            .bearer_auth(admin_token)
            .json(&channel_payload)
            .send()
            .await
            .context("Failed to create channel")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to create channel: {} - {}", status, body);
        }

        let channel: serde_json::Value = response.json().await?;
        let channel_id = channel["id"]
            .as_str()
            .context("No id in channel response")?
            .to_string();

        Ok(channel_id)
    }

    async fn create_bot(
        base_url: &str,
        admin_token: &str,
        team_id: &str,
        bot_name: &str,
    ) -> Result<(String, String)> {
        let client = reqwest::Client::new();

        let bot_payload = serde_json::json!({
            "username": bot_name,
            "display_name": format!("{} Bot", bot_name),
            "description": format!("Test bot {}", bot_name),
        });

        let response = client
            .post(format!("{}/api/v4/bots", base_url))
            .bearer_auth(admin_token)
            .json(&bot_payload)
            .send()
            .await
            .context("Failed to create bot")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to create bot: {} - {}", status, body);
        }

        let bot: serde_json::Value = response.json().await?;
        let bot_user_id = bot["user_id"]
            .as_str()
            .context("No user_id in bot response")?
            .to_string();

        let member_payload = serde_json::json!({
            "team_id": team_id,
            "user_id": bot_user_id,
        });

        client
            .post(format!("{}/api/v4/teams/{}/members", base_url, team_id))
            .bearer_auth(admin_token)
            .json(&member_payload)
            .send()
            .await
            .context("Failed to add bot to team")?;

        let token_payload = serde_json::json!({
            "description": format!("Test bot {} token", bot_name),
        });

        let token_response = client
            .post(format!("{}/api/v4/users/{}/tokens", base_url, bot_user_id))
            .bearer_auth(admin_token)
            .json(&token_payload)
            .send()
            .await
            .context("Failed to create bot token")?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().await?;
            anyhow::bail!("Failed to create bot token: {} - {}", status, body);
        }

        let token_data: serde_json::Value = token_response.json().await?;
        let token = token_data["token"]
            .as_str()
            .context("No token in token response")?
            .to_string();

        Ok((token, bot_user_id))
    }

    async fn add_user_to_channel(
        base_url: &str,
        admin_token: &str,
        channel_id: &str,
        user_id: &str,
    ) -> Result<()> {
        let client = reqwest::Client::new();

        let member_payload = serde_json::json!({
            "user_id": user_id,
        });

        let response = client
            .post(format!(
                "{}/api/v4/channels/{}/members",
                base_url, channel_id
            ))
            .bearer_auth(admin_token)
            .json(&member_payload)
            .send()
            .await
            .context("Failed to add user to channel")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Failed to add user to channel: {} - {}", status, body);
        }

        Ok(())
    }
}
