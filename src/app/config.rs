use crate::app::Cosmicding;
use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry},
    iced::Subscription,
    theme, Application,
};
use serde::{Deserialize, Serialize};
use std::any::TypeId;

pub const CONFIG_VERSION: u64 = 1;

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct CosmicConfig {
    pub app_theme: AppTheme,
    pub sort_option: SortOption,
    pub items_per_page: u8,
    pub enable_favicons: bool,
}

impl CosmicConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(Cosmicding::APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> CosmicConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                CosmicConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    log::info!("errors loading config: {errs:?}");
                    config
                })
            }
            None => CosmicConfig::default(),
        }
    }

    pub fn subscription() -> Subscription<cosmic_config::Update<Self>> {
        struct ConfigSubscription;
        cosmic_config::config_subscription(
            TypeId::of::<ConfigSubscription>(),
            Cosmicding::APP_ID.into(),
            CONFIG_VERSION,
        )
    }
}

impl Default for CosmicConfig {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            sort_option: SortOption::BookmarksDateNewest,
            items_per_page: 10,
            enable_favicons: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Default)]
pub enum SortOption {
    #[default]
    BookmarksDateNewest,
    BookmarksDateOldest,
    BookmarkAlphabeticalAscending,
    BookmarkAlphabeticalDescending,
}
