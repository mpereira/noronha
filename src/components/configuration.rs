use std::env;
use std::sync::{RwLock, RwLockWriteGuard};

use config::{self, Config};
use state::Storage;

static CONFIGURATION_FILE_ENV_VARIABLE: &'static str =
    "NORONHA_CONFIGURATION_FILE";

static DEFAULT_CONFIGURATION_FILE: &'static str = "config/Noronha.toml";

lazy_static! {
    static ref STATE: Storage<RwLock<Configuration>> = Storage::new();
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Configuration {
    pub cluster_name: String,
    pub node_name: String,
    pub bind_host: String,
    pub publish_host: String,
    pub cluster_peers: Vec<String>,
    pub http_resources_port: u32,
    pub http_resources_workers: usize,
    pub http_transport_port: u32,
    pub http_transport_workers: usize,
    pub http_transport_pinger_connect_timeout: u64,
    pub http_transport_pinger_schedule: u64,
    pub log_config_file: String,
}

impl Configuration {
    pub fn initialize() -> Self {
        let mut settings = Config::default();

        let config = settings
            .merge(config::File::with_name(&Self::configuration_file()))
            .unwrap()
            .clone();

        let configuration: Self = config.try_into().unwrap();

        STATE.set(RwLock::new(configuration));

        Self::read()
    }

    pub fn read() -> Self {
        STATE.get().read().unwrap().clone()
    }

    pub fn write() -> RwLockWriteGuard<'static, Self> {
        STATE.get().write().unwrap()
    }

    fn configuration_file() -> String {
        match env::var(CONFIGURATION_FILE_ENV_VARIABLE) {
            Ok(value) => {
                println!("Using configuration file from ENV: {}", value);
                value
            }
            Err(_error) => {
                println!(
                    "Using default configuration file: {}",
                    DEFAULT_CONFIGURATION_FILE
                );
                DEFAULT_CONFIGURATION_FILE.to_string()
            }
        }
    }
}
