use crate::engine::camera::viewport_camera::{ViewportCamera, camera_controller};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::engine::core::app_state::FpsText;
use crate::rpc::web_rpc::WebRpcInterface;

pub fn fps_notification_system(
    mut rpc_interface: ResMut<WebRpcInterface>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_send_time: Local<f32>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Send FPS every 0.5 seconds
    if current_time - *last_send_time >= 0.5 {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                rpc_interface.send_notification(
                    "fps_update",
                    serde_json::json!({
                        "fps": value as f32
                    }),
                );
                *last_send_time = current_time;
            }
        }
    }
}

pub fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {value:.1}");
            }
        }
    }
}
