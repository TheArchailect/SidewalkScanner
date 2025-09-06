use crate::tools::tool_manager::{ToolSelectionEvent, ToolSelectionSource, ToolType};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

/// JSON-RPC 2.0 notification structure for one-way communication.
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

            // Attempt JSON parsing to validate RPC format before queuing.
            if message_str.contains("jsonrpc") {
                if let Ok(mut queue) = queue_clone.lock() {
                    queue.push(message_str);
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

// #[cfg(not(target_arch = "wasm32"))]
// fn setup_message_listener(_commands: Commands) {
//     // No-op for non-WASM targets.
// }

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
) {
    for event in events.read() {
        // Send debug notification to frontend.
        rpc_interface.send_notification(
            "debug_message",
            serde_json::json!({
                "message": format!("Received RPC: {}", event.content)
            }),
        );

        match serde_json::from_str::<RpcRequest>(&event.content) {
            Ok(request) => {
                rpc_interface.send_notification(
                    "debug_message",
                    serde_json::json!({
                        "message": format!("Processing method: {}", request.method)
                    }),
                );

                if let Some(response) = handle_rpc_request(&request, &diagnostics, &mut tool_events)
                {
                    rpc_interface.queue_response(response);
                }
            }
            Err(parse_error) => {
                rpc_interface.send_notification(
                    "debug_message",
                    serde_json::json!({
                        "message": format!("Parse error: {}", parse_error)
                    }),
                );
            }
        }
    }
}

/// Handle individual RPC request and generate response based on method.
fn handle_rpc_request(
    request: &RpcRequest,
    diagnostics: &DiagnosticsStore,
    tool_events: &mut EventWriter<ToolSelectionEvent>,
) -> Option<RpcResponse> {
    // Only generate responses for requests with IDs (notifications have no ID).
    let id = request.id.clone()?;

    let result = match request.method.as_str() {
        "tool_selection" => handle_tool_selection(&request.params, tool_events),
        "get_fps" => handle_get_fps(diagnostics),
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
) -> Result<serde_json::Value, RpcError> {
    #[derive(serde::Deserialize)]
    struct ToolSelectionParams {
        tool: String,
    }

    let tool_params = serde_json::from_value::<ToolSelectionParams>(params.clone())
        .map_err(|_| RpcError::invalid_params("Expected 'tool' parameter"))?;

    // Convert string to tool type using generic mapping.
    let tool_type = ToolType::from_string(&tool_params.tool)
        .ok_or_else(|| RpcError::invalid_params(&format!("Unknown tool: {}", tool_params.tool)))?;

    // Dispatch tool selection event for processing by tool manager.
    tool_events.write(ToolSelectionEvent {
        tool_type,
        source: ToolSelectionSource::Rpc,
    });

    info!("Tool selection event dispatched: {:?}", tool_type);

    Ok(serde_json::json!({
        "success": true,
        "active_tool": tool_params.tool
    }))
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
