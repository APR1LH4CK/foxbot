use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub bot: BotConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub application_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub owners: Vec<u64>,
    pub embed_color: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Ok(config) = Self::from_env() {
            return Ok(config);
        }

        Self::from_file("config.toml")
    }

    fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let token = std::env::var("DISCORD_TOKEN")
            .context("DISCORD_TOKEN environment variable not found")?;

        let application_id: u64 = std::env::var("APPLICATION_ID")
            .context("APPLICATION_ID environment variable not found")?
            .parse()
            .context("APPLICATION_ID must be a valid u64")?;

        let owners: Vec<u64> = std::env::var("BOT_OWNERS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        let embed_color = std::env::var("EMBED_COLOR").unwrap_or_else(|_| "#dea584".to_string());

        Ok(Config {
            discord: DiscordConfig {
                token,
                application_id,
            },
            bot: BotConfig {
                owners,
                embed_color,
            },
        })
    }

    fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse config file: {}", path))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            discord: DiscordConfig {
                token: String::new(),
                application_id: 0,
            },
            bot: BotConfig {
                owners: vec![],
                embed_color: "#dea584".to_string(),
            },
        }
    }
}
