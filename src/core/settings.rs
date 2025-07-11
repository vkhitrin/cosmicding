use crate::core::i18n;

pub fn settings() -> cosmic::app::Settings {
    cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    )
    // NOTE: (vkhitrin) An example of client decorations enabled.
    //       Useless as long as we render the native
    //       libcosmic menu
    // #[cfg(target_os = "macos")]
    // {
    //     cosmic::app::Settings::default()
    //         .size_limits(
    //             cosmic::iced::Limits::NONE
    //                 .min_width(360.0)
    //                 .min_height(180.0),
    //         )
    //         .client_decorations(false)
    //         .resizable(None)
    // }
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
    pretty_env_logger::init_timed();
}
