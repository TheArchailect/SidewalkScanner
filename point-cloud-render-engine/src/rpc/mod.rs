//! JSON-RPC 2.0 communication layer for React frontend integration.
//!
//! Implements bidirectional messaging between Bevy engine and React UI via
//! iframe postMessage, supporting both request-response and notification patterns.
//!
//! ## Architecture
//!
//! The RPC system uses standard JSON-RPC 2.0 protocol with:
//! - **Requests**: Expect responses with matching IDs
//! - **Notifications**: One-way messages without responses
//! - **Responses**: Reply to requests with results or errors
//!
//! ## Message Flow
//!
//! ```text
//! React (Parent Window)  <──postMessage──>  Bevy (iframe)
//!        │                                        │
//!        ├─ Request (with ID) ──────────────────> │
//!        │                                        ├─ Process request
//!        │ <───────────────── Response (with ID) ─┤
//!        │                                        │
//!        │ <────────── Notification (no ID) ─────┤
//! ```
//!
//! ## Adding New RPC Methods
//!
//! ### 1. Define Request Handler
//!
//! Add a new method case in `handle_rpc_request()`:
//!
//! ```rust,ignore
//! fn handle_rpc_request(request: &RpcRequest, ...) -> Option<RpcResponse> {
//!     let result = match request.method.as_str() {
//!         "your_method_name" => handle_your_method(&request.params, ...),
//!         // ... existing methods
//!         _ => return Some(create_error_response(id, -32601, "Method not found", None)),
//!     };
//!     // ... response creation
//! }
//! ```
//!
//! ### 2. Implement Handler Function
//!
//! ```rust,ignore
//! fn handle_your_method(
//!     params: &Value,
//!     // ... required resources
//! ) -> Result<Value, RpcError> {
//!     // Deserialize parameters
//!     #[derive(Deserialize)]
//!     struct YourParams {
//!         field: String,
//!     }
//!
//!     let parsed = serde_json::from_value::<YourParams>(params.clone())
//!         .map_err(|_| RpcError::invalid_params("Expected 'field' parameter"))?;
//!
//!     // Process logic here
//!
//!     // Return success response
//!     Ok(json!({
//!         "success": true,
//!         "result": parsed.field
//!     }))
//! }
//! ```
//!
//! ### 3. Call From React
//!
//! ```typescript
//! // Request-response pattern
//! const response = await window.postMessage({
//!   jsonrpc: "2.0",
//!   method: "your_method_name",
//!   params: { field: "value" },
//!   id: 1
//! }, "*");
//!
//! // Notification pattern (no response expected)
//! window.postMessage({
//!   jsonrpc: "2.0",
//!   method: "your_notification",
//!   params: { data: "value" }
//! }, "*");
//! ```
//!
//! ## Sending Notifications from Bevy
//!
//! Use `WebRpcInterface::send_notification()` to push updates to React:
//!
//! ```rust,ignore
//! fn your_system(mut rpc: ResMut<WebRpcInterface>) {
//!     rpc.send_notification("event_name", json!({
//!         "data": "value",
//!         "timestamp": 123456
//!     }));
//! }
//! ```
//!
//! ## Error Handling
//!
//! Standard JSON-RPC 2.0 error codes:
//! - `-32600`: Invalid request
//! - `-32601`: Method not found
//! - `-32602`: Invalid params
//! - `-32603`: Internal error
//!
//! ## Existing Methods
//!
//! ### Tool Management
//! - `tool_selection`: Activate polygon/measure/asset tools
//! - `clear_tool`: Deactivate current tool
//!
//! ### Asset Operations
//! - `get_available_assets`: List all assets from manifest
//! - `get_asset_categories`: Retrieve asset category structure
//! - `select_asset`: Choose asset for placement
//! - `place_asset_at_position`: Place selected asset at coordinates
//!
//! ### Polygon Operations
//! - `get_classification_categories`: Get point cloud class types
//! - `hide_points_in_polygon`: Queue hide operation with mask filters
//! - `reclassify_points_in_polygon`: Queue reclassification with target class
//! - `set_hover_object_id`: Update hover highlight for object ID
//!
//! ### Diagnostics
//! - `get_fps`: Retrieve current frame rate
//!
//! ### Render Control
//! - `render_mode_changed`: Switch between RGB/Original/Modified/Connectivity views

/// JSON-RPC 2.0 bidirectional communication system for React integration.
///
/// Handles request-response patterns, notifications, and WASM message listeners.
pub mod web_rpc;
