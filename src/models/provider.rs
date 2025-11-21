use cosmic::widget;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Provider {
    Cosmicding,
    Linkding,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Cosmicding => write!(f, "cosmicding"),
            Provider::Linkding => write!(f, "linkding"),
        }
    }
}

impl Provider {
    pub fn from_str(s: &str) -> Self {
        match s {
            "cosmicding" => Provider::Cosmicding,
            "linkding" => Provider::Linkding,
            _ => Provider::Linkding,
        }
    }

    pub fn svg_icon(&self) -> widget::icon::Handle {
        match self {
            Provider::Linkding => {
                widget::icon::from_svg_bytes(include_bytes!("../../res/icons/linkding-logo.svg"))
            }
            Provider::Cosmicding => widget::icon::from_svg_bytes(include_bytes!(
                "../../res/icons/hicolor/scalable/apps/com.vkhitrin.cosmicding.svg"
            )),
        }
    }
}
