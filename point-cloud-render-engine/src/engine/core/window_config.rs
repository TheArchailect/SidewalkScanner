use bevy::prelude::*;
use bevy::window::PresentMode;

pub fn create_window_config() -> Window {
    #[cfg(target_arch = "wasm32")]
    {
        Window {
            canvas: Some("#bevy".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            present_mode: PresentMode::AutoVsync,
            ..default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Window {
            present_mode: PresentMode::AutoVsync,
            ..default()
        }
    }
}
