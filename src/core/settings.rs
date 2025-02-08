use crate::core::i18n;

pub fn settings() -> cosmic::app::Settings {
    cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    )
}

pub fn flags() -> crate::app::Flags {
    crate::app::Flags {
        config_handler: crate::app::config::CosmicConfig::config_handler(),
        config: crate::app::config::CosmicConfig::config(),
    }
}

pub fn init() {
    i18n::localize();

    std::env::set_var("RUST_LOG", "warn");
    pretty_env_logger::init();
    log::info!("such information");
}
