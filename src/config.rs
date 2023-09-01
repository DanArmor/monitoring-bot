use log::{debug, error, info};
use mobot::BotState;
use serde::Deserialize;

// Config to keep secrets and stuff
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    // Telegram bot token
    pub telegram_bot_token: String,
    // Data about NijiEN waves
    pub admins: Vec<i64>,
    // Addr to bind to
    pub server_addr: String
}

// Bot state, containts config data and pool of connections
#[derive(Debug, Clone, Default, BotState)]
pub struct MelatoninBotState {
    // App config
    config: Config,
}

impl MelatoninBotState {
    pub fn new(config: Config) -> Self {
        MelatoninBotState {
            config: config,
        }
    }
    // Get telegram bot token
    pub fn get_telegram_bot_token(&self) -> String {
        self.config.telegram_bot_token.clone()
    }
    pub fn get_admins(&self) -> &Vec<i64> {
        &self.config.admins
    }
    pub fn get_server_addr(&self) -> String {
        self.config.server_addr.clone()
    }
}