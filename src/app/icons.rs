use cosmic::widget::{self, icon::Handle};
#[cfg(target_os = "macos")]
use std::env;

// NOTE: Consider caching icons in the future
pub fn load_icon(icon: &str) -> Handle {
    // On Linux, we use the XDG desktop icons
    #[cfg(target_os = "linux")]
    {
        widget::icon::from_name(icon).handle()
    }

    // On macOS, we bundle images as part of the application
    #[cfg(target_os = "macos")]
    {
        let binary_path = env::current_exe().expect("Failed to get executable path");

        let resources_dir = binary_path
            .parent()
            .and_then(|p| p.parent())
            .expect("Failed to find app bundle path");

        let icon_path = resources_dir
            .join("Resources")
            .join("icons")
            .join(format!("{icon}.svg"));

        widget::icon::from_path(icon_path)
    }
}
