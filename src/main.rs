mod app;
mod config;
mod db;
mod http;
mod i18n;
mod menu;
mod models;
mod nav;
mod pages;
mod key_binds;

use crate::config::{Config, CONFIG_VERSION};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use app::{Flags, APPID};

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    let settings = cosmic::app::Settings::default();
    let (config_handler, config) = match cosmic_config::Config::new(APPID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            (None, Config::default())
        }
    };
    let flags = Flags {
        config_handler,
        config,
    };
    cosmic::app::run::<app::Cosmicding>(settings, flags)
}
