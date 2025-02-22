mod app;
mod core;
mod db;
mod http;
mod models;
mod pages;
mod style;
mod utils;

use core::settings;

fn main() -> cosmic::iced::Result {
    settings::init();
    cosmic::app::run::<app::Cosmicding>(settings::settings(), settings::flags())
}
