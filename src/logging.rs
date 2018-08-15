use log4rs;

pub struct Logging;

impl Logging {
    pub fn start(log_config_file: &str) -> () {
        log4rs::init_file(log_config_file, Default::default()).unwrap();
    }
}
