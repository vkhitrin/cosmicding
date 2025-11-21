mod app;
mod core;
mod db;
mod models;
mod pages;
mod provider;
mod style;
mod utils;
mod widgets;

use core::settings;

fn main() -> cosmic::iced::Result {
    settings::init();
    cosmic::app::run::<app::Cosmicding>(settings::settings(), settings::flags())
}
