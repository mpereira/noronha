use config::{self, Config};

use state::CONFIGURATION;

pub type Configuration = Config;

pub fn new() -> Config {
    Config::default()
}

pub fn start(configuration_file: &str) -> () {
    let mut configuration = CONFIGURATION.write().unwrap();
    configuration
        .merge(config::File::with_name(configuration_file))
        .unwrap();
}
