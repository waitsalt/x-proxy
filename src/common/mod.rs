pub mod config;
pub mod logger;
pub mod util;

pub fn init() {
    logger::init();
}
