use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Enumeration of available tools in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Polygon,
    Measure,
}

impl ToolType {
    /// Convert string identifier to tool type for RPC compatibility.
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "polygon" => Some(Self::Polygon),
            "measure" => Some(Self::Measure),
            _ => None,
        }
    }

    /// Convert tool type to string identifier for frontend communication.
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Polygon => "polygon",
            Self::Measure => "knife",
        }
    }
}

/// Resource tracking the currently active tool and tool-specific state.
#[derive(Resource)]
pub struct ToolManager {
    /// Currently selected tool, if any.
    active_tool: Option<ToolType>,
}

impl Default for ToolManager {
    fn default() -> Self {
        Self { active_tool: None }
    }
}

impl ToolManager {
    /// Activate specified tool, deactivating previous tool if necessary.
    pub fn activate_tool(&mut self, tool_type: ToolType) -> bool {
        // Check if tool is already active to avoid redundant operations.
        if self.active_tool == Some(tool_type) {
            return false; // No change needed.
        }

        // Deactivate current tool before switching.
        self.active_tool = Some(tool_type);
        info!("Tool manager activated: {}", tool_type.to_string());
        true // Tool changed.
    }

    /// Deactivate currently active tool and clear selection.
    pub fn deactivate_current_tool(&mut self) -> Option<ToolType> {
        let previous = self.active_tool.take();
        if let Some(tool) = previous {
            info!("Tool manager deactivated: {}", tool.to_string());
        }
        previous
    }

    /// Get currently active tool type.
    pub fn active_tool(&self) -> Option<ToolType> {
        self.active_tool
    }

    /// Check if specific tool is currently active.
    pub fn is_tool_active(&self, tool_type: ToolType) -> bool {
        self.active_tool == Some(tool_type)
    }
}

/// Event fired when tool selection changes via RPC or keyboard shortcuts.
#[derive(Event)]
pub struct ToolSelectionEvent {
    pub tool_type: ToolType,
    pub source: ToolSelectionSource,
}

/// Event fired when polygon actions are requested via RPC.
#[derive(Event)]
pub struct PolygonActionEvent {
    pub action: PolygonAction,
}

/// Available polygon actions that can be triggered remotely.
#[derive(Debug, Clone, Copy)]
pub enum PolygonAction {
    Complete, // Finish current polygon (equivalent to Shift key).
    Clear,    // Clear current polygon (equivalent to 'O' key).
    ClearAll, // Clear all polygons (equivalent to 'I' key).
}

/// Source of tool selection for debugging and conditional logic.
#[derive(Debug, Clone, Copy)]
pub enum ToolSelectionSource {
    Rpc,
    Keyboard,
}

/// System handling tool selection events with proper state coordination.
pub fn handle_tool_selection_events(
    mut events: EventReader<ToolSelectionEvent>,
    mut tool_manager: ResMut<ToolManager>,
    mut polygon_tool: ResMut<crate::tools::polygon::PolygonTool>,
    mut rpc_interface: ResMut<crate::rpc::web_rpc::WebRpcInterface>,
) {
    for event in events.read() {
        // Update tool manager state first.
        let tool_changed = tool_manager.activate_tool(event.tool_type);

        if !tool_changed {
            continue; // Tool already active, skip redundant operations.
        }

        // Deactivate all tools first to ensure clean state.
        polygon_tool.set_active(false);

        // Activate the requested tool.
        match event.tool_type {
            ToolType::Polygon => {
                polygon_tool.set_active(true);

                info!("Polygon tool activated via {:?}", event.source);
                println!(
                    "Polygon classification tool activated. Current class: {}",
                    polygon_tool.current_class
                );
                println!("Left click to add points, Complete via RPC or Shift key");

                // Send confirmation to frontend with current tool state.
                rpc_interface.send_notification(
                    "tool_state_changed",
                    serde_json::json!({
                        "tool": "polygon",
                        "active": true,
                        "current_class": polygon_tool.current_class
                    }),
                );
            }
            ToolType::Measure => {
                info!(
                    "Measure tool activated via {:?} (placeholder)",
                    event.source
                );

                // Send confirmation to frontend.
                rpc_interface.send_notification(
                    "tool_state_changed",
                    serde_json::json!({
                        "tool": "measure",
                        "active": true
                    }),
                );
            }
        }
    }
}

/// System handling polygon-specific actions triggered via RPC.
pub fn handle_polygon_action_events(
    mut events: EventReader<PolygonActionEvent>,
    mut polygon_tool: ResMut<crate::tools::polygon::PolygonTool>,
    tool_manager: Res<ToolManager>,
    mut rpc_interface: ResMut<crate::rpc::web_rpc::WebRpcInterface>,
) {
    // Only process polygon actions when polygon tool is active.
    if !tool_manager.is_tool_active(ToolType::Polygon) {
        return;
    }

    for event in events.read() {
        match event.action {
            PolygonAction::Complete => {
                if polygon_tool.current_polygon.len() >= 3 {
                    // Set completion flag - polygon_tool_system will handle the rest.
                    polygon_tool.is_completed = true;

                    info!("Polygon completion triggered via RPC");

                    // Send acknowledgment to frontend.
                    rpc_interface.send_notification(
                        "polygon_action_completed",
                        serde_json::json!({
                            "action": "complete",
                            "success": true
                        }),
                    );
                } else {
                    warn!("Cannot complete polygon: need at least 3 points");

                    rpc_interface.send_notification(
                        "polygon_action_completed",
                        serde_json::json!({
                            "action": "complete",
                            "success": false,
                            "reason": "Need at least 3 points"
                        }),
                    );
                }
            }
            PolygonAction::Clear => {
                polygon_tool.current_polygon.clear();
                polygon_tool.preview_point = None;
                polygon_tool.is_completed = false;

                info!("Current polygon cleared via RPC");

                rpc_interface.send_notification(
                    "polygon_action_completed",
                    serde_json::json!({
                        "action": "clear",
                        "success": true
                    }),
                );
            }
            PolygonAction::ClearAll => {
                polygon_tool.current_polygon.clear();
                polygon_tool.preview_point = None;
                polygon_tool.is_completed = false;

                info!("All polygons cleared via RPC");

                rpc_interface.send_notification(
                    "polygon_action_completed",
                    serde_json::json!({
                        "action": "clear_all",
                        "success": true
                    }),
                );
            }
        }
    }
}

/// System handling keyboard shortcuts for tool selection (native builds only).
#[cfg(not(target_arch = "wasm32"))]
pub fn handle_tool_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tool_events: EventWriter<ToolSelectionEvent>,
) {
    // Map keyboard shortcuts to tool types.
    if keyboard.just_pressed(KeyCode::KeyP) {
        tool_events.send(ToolSelectionEvent {
            tool_type: ToolType::Polygon,
            source: ToolSelectionSource::Keyboard,
        });
    }

    if keyboard.just_pressed(KeyCode::KeyK) {
        tool_events.send(ToolSelectionEvent {
            tool_type: ToolType::Measure,
            source: ToolSelectionSource::Keyboard,
        });
    }
}

/// Placeholder system for WASM builds where keyboard shortcuts are disabled.
#[cfg(target_arch = "wasm32")]
pub fn handle_tool_keyboard_shortcuts() {
    // No keyboard shortcuts in WASM builds - tools controlled via RPC only.
}
