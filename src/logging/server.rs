use log::{info, error};

pub fn log_info(message: String) {
    info!("{}", message);
}

pub fn log_error(message: String) {
    error!("{}", message);
}