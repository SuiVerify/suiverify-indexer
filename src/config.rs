use std::env;

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub enable_detailed_logs: bool,
    pub log_level: String,
    pub log_events: bool,
}

impl LogConfig {
    pub fn from_env() -> Self {
        Self {
            enable_detailed_logs: env::var("ENABLE_DETAILED_LOGS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            log_events: env::var("LOG_EVENTS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }

    pub fn should_log_events(&self) -> bool {
        self.log_events && self.enable_detailed_logs
    }

    pub fn should_log_detailed(&self) -> bool {
        self.enable_detailed_logs
    }
}
