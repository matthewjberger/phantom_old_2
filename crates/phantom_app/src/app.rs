use phantom_dependencies::{env_logger, log};

pub fn run() {
    env_logger::init();
    log::info!("This is a phantom app!")
}
