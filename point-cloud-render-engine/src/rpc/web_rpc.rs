use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::systems::render_mode::{RenderMode, RenderModeState};
use crate::tools::asset_manager::PlaceAssetBoundState;
use crate::tools::tool_manager::{
    AssetPlacementAction, AssetPlacementEvent, ToolSelectionEvent, ToolSelectionSource, ToolType,
    ClearToolEvent,
};
use crate::tools::polygon::{PolygonHideRequestEvent, PolygonReclassifyRequestEvent}; // Polygon import module path
use serde_json::{Value, json};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
//use serde_json::{json, Value};

/// Polygon RPC DTOs
#[derive(Debug, Deserialize)]
struct SourceItem {
    category_id: String,
    item_id: String,
}

#[derive(Debug, Deserialize)]
struct HideParams {
    #[serde(default)]
    source_items: Vec<SourceItem>,
}

#[derive(Debug, Deserialize)]
struct ReclassifyParams {
    #[serde(default)]
    source_items: Vec<SourceItem>,
    target_category_id: String,
    target_item_id: String,
}

#[derive(Debug, Serialize)]
struct PolygonOperationResult {
    success: bool,
    points_affected:u64,
    message: String,
}



#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
use web_sys::{MessageEvent, window};

/// JSON-RPC 2.0 request structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC notification structure for one-way communication.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// JSON-RPC error structure following specification.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Resource managing bidirectional RPC communication between React and Bevy.
/// Handles both request-response patterns and notification broadcasting.
#[derive(Resource, Default)]
pub struct WebRpcInterface {
    outgoing_notifications: Vec<RpcNotification>,
    outgoing_responses: Vec<RpcResponse>,
}

impl WebRpcInterface {
    /// Send notification to React frontend without expecting response.
    pub fn send_notification(&mut self, method: &str, params: serde_json::Value) {
        self.outgoing_notifications.push(RpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        });
    }

    /// Queue response for transmission to React frontend.
    fn queue_response(&mut self, response: RpcResponse) {
        self.outgoing_responses.push(response);
    }
}

/// Plugin establishing WebRPC communication layer for iframe-based deployment.
pub struct WebRpcPlugin;

impl Plugin for WebRpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WebRpcInterface>()
            .add_event::<IncomingRpcMessage>()
            .add_systems(
                Update,
                (
                    process_incoming_messages,
                    handle_rpc_messages,
                    send_outgoing_messages,
                )
                    .chain(),
            );

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Startup, setup_message_listener);
    }
}

#[cfg(target_arch = "wasm32")]
fn setup_message_listener(mut commands: Commands) {
    use std::sync::Arc;
    use std::sync::Mutex;

    // Thread-safe message queue for cross-thread communication.
    let message_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let queue_clone = message_queue.clone();

    let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
        // Filter messages to ensure they contain string data.
        if let Ok(data) = event.data().dyn_into::<js_sys::JsString>() {
            let message_str: String = data.into();

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message_str) {
                if json.get("jsonrpc").is_some() {
                    if message_str.contains("get_available_assets") {
                        web_sys::console::log_1(
                            &"[DEBUG] get_available_assets received and queued".into(),
                        );
                    }
                    if let Ok(mut queue) = queue_clone.lock() {
                        queue.push(message_str);
                    }
                }
            }
        }
    }) as Box<dyn FnMut(MessageEvent)>);

    if let Some(window) = window() {
        window
            .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
            .expect("Failed to register message listener");
    }

    // Prevent closure from being dropped by transferring ownership to JS.
    closure.forget();
    commands.insert_resource(MessageQueue(message_queue));
}

/// Resource wrapping thread-safe message queue for WASM event handling.
#[derive(Resource)]
struct MessageQueue(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

/// Event representing incoming RPC message from React frontend.
#[derive(Event)]
struct IncomingRpcMessage {
    content: String,
}

fn process_incoming_messages(
    message_queue: Option<Res<MessageQueue>>,
    mut message_events: EventWriter<IncomingRpcMessage>,
) {
    let Some(queue_res) = message_queue else {
        return;
    };

    let messages = if let Ok(mut queue) = queue_res.0.lock() {
        std::mem::take(&mut *queue)
    } else {
        Vec::new()
    };

    // Write events using the non-deprecated method.
    for message_str in messages {
        message_events.write(IncomingRpcMessage {
            content: message_str,
        });
    }
}

fn handle_rpc_messages(
    mut events: EventReader<IncomingRpcMessage>,
    diagnostics: Res<DiagnosticsStore>,
    mut rpc_interface: ResMut<WebRpcInterface>,
    mut tool_events: EventWriter<ToolSelectionEvent>,
    mut asset_placement_events: EventWriter<AssetPlacementEvent>,
    mut render_state: ResMut<RenderModeState>,
    mut place_asset_state: ResMut<PlaceAssetBoundState>,
    assets: Res<PointCloudAssets>,
    manifests: Res<Assets<SceneManifest>>,
    mut clear_events: EventWriter<ClearToolEvent>,
    mut polygon_hide_events: EventWriter<PolygonHideRequestEvent>,              // Polygon event writer to handle Hide request
    mut polygon_reclassify_events: EventWriter<PolygonReclassifyRequestEvent>   // Polygon event writer to handle Reclassify request
) {
    for event in events.read() {
        // Parse as generic JSON first to check for 'id' field
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&event.content) {
            // Check if it has an 'id' field to determine if it's a request or notification
            if json_value.get("id").is_some() {
                // Has ID field - try parsing as request
                if let Ok(request) = serde_json::from_str::<RpcRequest>(&event.content) {
                    if let Some(response) = handle_rpc_request(
                        &request,
                        &diagnostics,
                        &mut tool_events,
                        &mut asset_placement_events,
                        &mut place_asset_state,
                        &assets,
                        &manifests,
                        &mut clear_events,
                        &mut polygon_hide_events,       // Polygon Hide event writer
                        &mut polygon_reclassify_events, // Polygon Reclassify writer
                    ) {
                        rpc_interface.queue_response(response);
                    }
                } else {
                    warn!("Failed to parse as RPC request: {}", event.content);
                }
            } else {
                // No ID field - try parsing as notification
                if let Ok(notification) = serde_json::from_str::<RpcNotification>(&event.content) {
                    handle_rpc_notification(&notification, &mut render_state);
                } else {
                    warn!("Failed to parse as RPC notification: {}", event.content);
                }
            }
        } else {
            warn!("Failed to parse as JSON: {}", event.content);
            rpc_interface.send_notification(
                "debug_message",
                serde_json::json!({
                    "message": format!("Parse error: invalid JSON")
                }),
            );
        }
    }
}

/// Handle individual RPC request and generate response based on method.
fn handle_rpc_request(
    request: &RpcRequest,
    diagnostics: &DiagnosticsStore,
    tool_events: &mut EventWriter<ToolSelectionEvent>,
    asset_placement_events: &mut EventWriter<AssetPlacementEvent>,
    place_asset_state: &mut ResMut<PlaceAssetBoundState>,
    assets: &Res<PointCloudAssets>,
    manifests: &Res<Assets<SceneManifest>>,
    clear_events: &mut EventWriter<ClearToolEvent>,
    polygon_hide_events: &mut EventWriter<PolygonHideRequestEvent>,             // Accept Polygon Hide writer
    polygon_reclassify_events: &mut EventWriter<PolygonReclassifyRequestEvent>  // Accept Polygon Reclassify writer
) -> Option<RpcResponse> {
    // Only generate responses for requests with IDs (notifications have no ID).
    let id = request.id.clone()?;

    let result = match request.method.as_str() {
        "tool_selection" => handle_tool_selection(&request.params, tool_events, clear_events),
        "clear_tool" => handle_clear_tool_request(clear_events),
        "get_fps" => handle_get_fps(diagnostics),
        "get_available_assets" => handle_get_available_assets(assets, manifests),
        "get_asset_categories" => handle_get_asset_categories(assets, manifests),
        "select_asset" => handle_select_asset(
            &request.params,
            place_asset_state,
            asset_placement_events,
            assets,
            manifests,
        ),
        "place_asset_at_position" => {
            handle_place_asset_at_position(&request.params, asset_placement_events)
        },
        // Polygon rpc
        //"get_classification_categories" => handle_get_classification_categories(assets, manifests),
        "hide_points_in_polygon" => handle_hide_points_in_polygon(&request.params, polygon_hide_events),
        "reclassify_points_in_polygon" => handle_reclassify_points_in_polygon(&request.params, polygon_reclassify_events),
        _ => {
            warn!("Unknown RPC method: {}", request.method);
            return Some(create_error_response(
                id,
                -32601,
                "Method not found",
                Some(serde_json::json!({"method": request.method})),
            ));
        }
    };

    match result {
        Ok(result_value) => Some(RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result_value),
            error: None,
            id: Some(id),
        }),
        Err(error) => Some(RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id: Some(id),
        }),
    }
}

/// Handle tool selection RPC method with parameter validation and event dispatch.
fn handle_tool_selection(
    params: &serde_json::Value,
    tool_events: &mut EventWriter<ToolSelectionEvent>,
    clear_events: &mut EventWriter<ClearToolEvent>,
) -> Result<serde_json::Value, RpcError> {
    #[derive(serde::Deserialize)]
    struct ToolSelectionParams {
        tool: Option<String>,
    }

    let tool_params = serde_json::from_value::<ToolSelectionParams>(params.clone())
        .map_err(|_| RpcError::invalid_params("Expected 'tool' parameter"))?;

    if tool_params
        .tool
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("none"))
        .unwrap_or(true)
    {
        clear_events.write(ClearToolEvent {
            source: ToolSelectionSource::Rpc,
        });
        info!("Tool cleared via RPC");
        return Ok(serde_json::json!({
            "success": true,
            "active_tool": "none"
        }));
    }

    // Convert string to tool type using generic mapping.
    let tool_str = tool_params.tool.unwrap();
    let tool_type = ToolType::from_string(&tool_str)
        .ok_or_else(|| RpcError::invalid_params(&format!("Unknown tool: {}", tool_str)))?;

    // Dispatch tool selection event for processing by tool manager.
    tool_events.write(ToolSelectionEvent {
        tool_type,
        source: ToolSelectionSource::Rpc,
    });

    info!("Tool selection event dispatched: {:?}", tool_type);

    Ok(serde_json::json!({
        "success": true,
        "active_tool": tool_str
    }))
}

fn handle_clear_tool_request(
    clear_events: &mut EventWriter<ClearToolEvent>,
) -> Result<serde_json::Value, RpcError> {
    clear_events.write(ClearToolEvent {
        source: ToolSelectionSource::Rpc,
    });
    Ok(serde_json::json!({ "success": true }))
}

/// Handle FPS retrieval with diagnostic system integration.
fn handle_get_fps(diagnostics: &DiagnosticsStore) -> Result<serde_json::Value, RpcError> {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps_diagnostic| fps_diagnostic.smoothed())
        .unwrap_or(0.0) as f32;

    Ok(serde_json::json!({
        "fps": fps
    }))
}

/// Handle getting available assets from the manifest.
fn handle_get_available_assets(
    assets: &Res<PointCloudAssets>,
    manifests: &Res<Assets<SceneManifest>>,
) -> Result<serde_json::Value, RpcError> {
    // Debug: Check if we have a manifest handle
    if assets.manifest.is_none() {
        return Err(RpcError::internal_error(
            "No manifest handle found in PointCloudAssets",
        ));
    }

    let manifest_handle = assets.manifest.as_ref().unwrap();

    // Debug: Check if the manifest is loaded
    let manifest = manifests.get(manifest_handle);
    if manifest.is_none() {
        return Err(RpcError::internal_error(
            "Manifest handle exists but manifest not loaded yet",
        ));
    }

    let manifest = manifest.unwrap();

    // Debug: Check if asset_atlas exists
    if manifest.asset_atlas.is_none() {
        return Err(RpcError::internal_error(
            "Manifest loaded but asset_atlas is None",
        ));
    }

    let atlas = manifest.asset_atlas.as_ref().unwrap();

    // Debug: Check how many assets we have
    let asset_count = atlas.assets.len();
    if asset_count == 0 {
        return Err(RpcError::internal_error(
            "Asset atlas exists but contains no assets",
        ));
    }

    // Log the asset names for debugging
    let asset_names: Vec<&String> = atlas.assets.iter().map(|a| &a.name).collect();
    info!("Found {} assets: {:?}", asset_count, asset_names);

    let asset_data: Vec<serde_json::Value> = atlas
        .assets
        .iter()
        .map(|asset| {
            serde_json::json!({
                "id": asset.name,
                "name": asset.name,
                "category": "assets",
                "point_count": asset.point_count,
                "uv_bounds": {
                    "uv_min": asset.uv_bounds.uv_min,
                    "uv_max": asset.uv_bounds.uv_max
                },
                "local_bounds": {
                    "min_x": asset.local_bounds.min_x,
                    "min_y": asset.local_bounds.min_y,
                    "min_z": asset.local_bounds.min_z,
                    "max_x": asset.local_bounds.max_x,
                    "max_y": asset.local_bounds.max_y,
                    "max_z": asset.local_bounds.max_z
                }
            })
        })
        .collect();

    info!("Returning {} assets to frontend", asset_data.len());
    Ok(serde_json::json!(asset_data))
}

/// Handle getting asset categories.
fn handle_get_asset_categories(
    assets: &Res<PointCloudAssets>,
    manifests: &Res<Assets<SceneManifest>>,
) -> Result<serde_json::Value, RpcError> {
    let manifest = assets
        .manifest
        .as_ref()
        .and_then(|h| manifests.get(h))
        .ok_or_else(|| RpcError::internal_error("Scene manifest not available"))?;

    // For now, return a simple default category structure
    // You may want to extend this based on your asset categorization needs
    let categories = vec![
        serde_json::json!({
            "id": "all",
            "name": "All Assets"
        }),
        serde_json::json!({
            "id": "assets",
            "name": "Assets"
        }),
    ];

    Ok(serde_json::json!(categories))
}

/// Handle asset selection.
fn handle_select_asset(
    params: &serde_json::Value,
    place_asset_state: &mut ResMut<PlaceAssetBoundState>,
    asset_placement_events: &mut EventWriter<AssetPlacementEvent>,
    assets: &Res<PointCloudAssets>,
    manifests: &Res<Assets<SceneManifest>>,
) -> Result<serde_json::Value, RpcError> {
    #[derive(serde::Deserialize)]
    struct SelectAssetParams {
        asset_id: String,
    }

    let select_params = serde_json::from_value::<SelectAssetParams>(params.clone())
        .map_err(|_| RpcError::invalid_params("Expected 'asset_id' parameter"))?;

    // Verify the asset exists in the manifest
    let manifest = assets
        .manifest
        .as_ref()
        .and_then(|h| manifests.get(h))
        .ok_or_else(|| RpcError::internal_error("Scene manifest not available"))?;

    let asset_exists = manifest
        .asset_atlas
        .as_ref()
        .map(|atlas| {
            atlas
                .assets
                .iter()
                .any(|asset| asset.name == select_params.asset_id)
        })
        .unwrap_or(false);

    if !asset_exists {
        return Err(RpcError::invalid_params(&format!(
            "Asset not found: {}",
            select_params.asset_id
        )));
    }

    // Update the selected asset state
    place_asset_state.selected_asset_name = Some(select_params.asset_id.clone());

    // Send asset selection event
    asset_placement_events.write(AssetPlacementEvent {
        action: AssetPlacementAction::SelectAsset,
        asset_id: Some(select_params.asset_id.clone()),
        position: None,
    });

    info!("Asset selected: {}", select_params.asset_id);

    // Find and return the selected asset data
    if let Some(atlas) = manifest.asset_atlas.as_ref() {
        if let Some(asset) = atlas
            .assets
            .iter()
            .find(|a| a.name == select_params.asset_id)
        {
            return Ok(serde_json::json!({
                "id": asset.name,
                "name": asset.name,
                "category": "assets",
                "point_count": asset.point_count,
                "uv_bounds": {
                    "uv_min": asset.uv_bounds.uv_min,
                    "uv_max": asset.uv_bounds.uv_max
                },
                "local_bounds": {
                    "min_x": asset.local_bounds.min_x,
                    "min_y": asset.local_bounds.min_y,
                    "min_z": asset.local_bounds.min_z,
                    "max_x": asset.local_bounds.max_x,
                    "max_y": asset.local_bounds.max_y,
                    "max_z": asset.local_bounds.max_z
                }
            }));
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "asset_id": select_params.asset_id
    }))
}

/// Handle placing asset at specific position.
fn handle_place_asset_at_position(
    params: &serde_json::Value,
    asset_placement_events: &mut EventWriter<AssetPlacementEvent>,
) -> Result<serde_json::Value, RpcError> {
    #[derive(serde::Deserialize)]
    struct PlaceAssetParams {
        x: f32,
        y: f32,
        z: f32,
    }

    let place_params = serde_json::from_value::<PlaceAssetParams>(params.clone())
        .map_err(|_| RpcError::invalid_params("Expected 'x', 'y', 'z' parameters"))?;

    let position = Vec3::new(place_params.x, place_params.y, place_params.z);

    // Send asset placement event
    asset_placement_events.write(AssetPlacementEvent {
        action: AssetPlacementAction::PlaceAtPosition,
        asset_id: None,
        position: Some(position),
    });

    info!("Asset placement requested at position: {:?}", position);

    Ok(serde_json::json!({
        "success": true,
        "position": [place_params.x, place_params.y, place_params.z]
    }))
}

/// Create standardized error response with optional data payload.
fn create_error_response(
    id: serde_json::Value,
    code: i32,
    message: &str,
    data: Option<serde_json::Value>,
) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(RpcError {
            code,
            message: message.to_string(),
            data,
        }),
        id: Some(id),
    }
}

/// Send queued notifications and responses to React frontend.
fn send_outgoing_messages(mut rpc_interface: ResMut<WebRpcInterface>) {
    // Send notifications first.
    for notification in rpc_interface.outgoing_notifications.drain(..) {
        send_message_to_parent(&notification);
    }

    // Send responses second to maintain order.
    for response in rpc_interface.outgoing_responses.drain(..) {
        send_message_to_parent(&response);
    }
}

/// Send serialized message to parent window (React frontend).
fn send_message_to_parent<T: Serialize>(message: &T) {
    #[cfg(target_arch = "wasm32")]
    {
        match serde_json::to_string(message) {
            Ok(json) => {
                if let Some(window) = window() {
                    if let Some(parent) = window.parent().ok().flatten() {
                        if let Err(e) = parent.post_message(&JsValue::from_str(&json), "*") {
                            error!("Failed to send message to parent: {:?}", e);
                        }
                    } else {
                        warn!("No parent window available for message transmission");
                    }
                } else {
                    error!("Window object not available");
                }
            }
            Err(e) => {
                error!("Failed to serialize message: {}", e);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // No-op for non-WASM targets.
        let _ = message;
    }
}

/// Standard RPC error codes and constructors.
impl RpcError {
    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: -32602,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self {
            code: -32603,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Handle RPC notifications
fn handle_rpc_notification(
    notification: &RpcNotification,
    render_state: &mut ResMut<RenderModeState>,
) {
    match notification.method.as_str() {
        "render_mode_changed" => {
            if let Some(mode_str) = notification.params.get("mode").and_then(|v| v.as_str()) {
                let new_mode = match mode_str {
                    "original" => RenderMode::OriginalClassification,
                    "modified" => RenderMode::ModifiedClassification,
                    "RGB" => RenderMode::RgbColour,
                    _ => {
                        warn!("Unknown render mode: {}", mode_str);
                        return;
                    }
                };
                render_state.current_mode = new_mode;
            }
        }
        _ => {
            warn!("Unknown RPC notification method: {}", notification.method);
        }
    }
}

/// Build polygon categories/items from Manieft/Asset Atlas
/// This mirrors AssetLibrary source; single "assets" category with all atlas entries
/*fn handle_get_classification_categories(
    assets: &Res<PointCloudAssets>,
    manifests: &Res<Assets<SceneManifest>>,
) -> Result<Value, RpcError> {
    let manifest_handle = assets.manifest.as_ref().unwrap();
    let manifest = manifests.get(manifest_handle).unwrap();
    let atlas = manifest.asset_atlas.as_ref().unwrap();

    let items: Vec<Value> = atlas
    .assets
    .iter()
    .map(|a| {
        json! ({
            "id": a.name,
            "name": a.name,
            "point_count": a.point_count,
        })
    })
    .collect();

    let total: u64 = items
    .iter()
    .map(|v| v.get("point_count").and_then(|n| n.as_u64()).unwrap_or(0))
    .sum();

    let categories = vec![json!({
        "id": "assets",
        "name": "Assets",
        "color": "#4aa3ff",
        "point_count": total,
        "items": items,
    })];
    Ok(json!(categories))
}*/

/// Parse + queue hide operation (engine hook comes next step).
fn handle_hide_points_in_polygon(
    params: &Value,
    polygon_hide_events: &mut EventWriter<PolygonHideRequestEvent>,
) -> Result<Value, RpcError> {
    let p: HideParams = serde_json::from_value(params.clone())
        .map_err(|_| RpcError::invalid_params(
            "Expected { source_items?: [{category_id,item_id}] }"))?;

    info!("[RPC] hide_points_in_polygon queued; filters={}", p.source_items.len());

    polygon_hide_events.write(PolygonHideRequestEvent {
        source_items: p.source_items
            .into_iter()
            .map(|s| (s.category_id, s.item_id))
            .collect()
    });

    let result = PolygonOperationResult {
        success: true,
        points_affected: 0,
        message: "Hide operation queued".to_string(),
    };

    Ok(serde_json::to_value(result).unwrap())
}

//Parse + queue reclassify operation (engine hook comes next step).
fn handle_reclassify_points_in_polygon(
    params: &Value,
    polygon_reclassify_events: &mut EventWriter<PolygonReclassifyRequestEvent>,
) -> Result<Value, RpcError> {
    let p: ReclassifyParams = serde_json::from_value(params.clone())
        .map_err(|_| RpcError::invalid_params(
            "Expected { source_items?:[], target_category_id, target_item_id }"
        ))?;

    info!(
        "[RPC] reclassify_points_in_polygon queued; filters={}, target=({},{})",
        p.source_items.len(), p.target_category_id, p.target_item_id
    );

    polygon_reclassify_events.write(PolygonReclassifyRequestEvent {
        source_items: p.source_items
            .into_iter()
            .map(|s| (s.category_id, s.item_id))
            .collect(),
        target: (p.target_category_id, p.target_item_id),
    });

    Ok(json!(PolygonOperationResult {
        success: true,
        points_affected: 0,
        message: "Reclassify operation queued".to_string(),
    }))

}
