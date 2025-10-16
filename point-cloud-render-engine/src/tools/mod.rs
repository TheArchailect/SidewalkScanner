//! Interactive tools for point cloud editing and measurement.
//!
//! Provides polygon-based reclassification, asset placement with instancing,
//! distance measurement, and point selection tools with unified tool manager
//! coordination and RPC integration for frontend control.
//!
//! ## Tool Manager Architecture
//!
//! The `ToolManager` resource maintains exclusive tool activation state:
//! - Only one tool can be active at a time
//! - Tools are activated via keyboard shortcuts (native) or RPC events (WASM)
//! - Deactivation clears tool-specific state and notifies frontend
//!
//! ### Tool Activation Flow
//!
//! ```text
//! Keyboard/RPC Input
//!   └─> ToolSelectionEvent
//!       └─> handle_tool_selection_events()
//!           ├─> Deactivate all tools
//!           ├─> Activate requested tool
//!           └─> Send RPC notification to frontend
//! ```
//!
//! ## Available Tools
//!
//! ### Polygon Tool (`ToolType::Polygon`)
//! - **Activation**: `P` key (native) or `tool_selection` RPC with `"polygon"`
//! - **Purpose**: Draw polygons for point reclassification or hiding operations
//! - **Workflow**:
//!   1. Left click adds polygon vertices with heightmap intersection
//!   2. Shift key (native) or RPC completion event finalises polygon
//!   3. Polygon resampled to uniform point spacing for GPU performance
//!   4. Compute shader applies classification changes within polygon bounds
//! - **Features**:
//!   - Mask filtering: Only affect specific class/object ID combinations
//!   - Hide mode: Set points to classification 254 (discarded in rendering)
//!   - Reclassify mode: Change points to target classification ID
//!   - Visual feedback with emissive vertex markers and edge lines
//!
//! ### Asset Placement Tool (`ToolType::AssetPlacement`)
//! - **Activation**: `A` key (native) or `tool_selection` RPC with `"assets"`
//! - **Purpose**: Place and manipulate instanced asset objects in scene
//! - **Workflow**:
//!   1. Select asset via RPC `select_asset` method
//!   2. Preview shows at mouse cursor with wireframe bounds
//!   3. Left click places instance at heightmap intersection
//!   4. Selected instances follow mouse, rotate with scroll wheel
//!   5. Delete key removes selected instances
//! - **Features**:
//!   - Real-time preview with heightmap-aware positioning
//!   - OBB raycasting for precise selection
//!   - Instanced rendering for GPU-efficient object placement
//!   - Persistent storage in `PlacedAssetInstances` resource
//!
//! ### Measure Tool (`ToolType::Measure`)
//! - **Activation**: `M` key (native) or `tool_selection` RPC with `"measure"`
//! - **Purpose**: Measure distances between two points on terrain
//! - **Workflow**:
//!   1. First click sets start point
//!   2. Mouse movement shows live preview with distance
//!   3. Second click completes measurement
//!   4. New measurement clears previous one
//! - **Features**:
//!   - Live distance updates sent to frontend via RPC
//!   - Heightmap-aware measurement along terrain surface
//!   - Visual feedback with coloured line segments
//!
//! ### Class Selection Tool
//! - **Activation**: `S` key (native only, no RPC integration)
//! - **Purpose**: Select individual points by connectivity class ID
//! - **Status**: Legacy tool, primarily for debugging connectivity data
//!
//! ## Cross-Platform Considerations
//!
//! ### Native Builds
//! - Keyboard shortcuts for tool activation and polygon operations
//! - Native UI panel for asset manager (collapsible side panel)
//! - Direct keyboard input for classification changes and clearing
//!
//! ### WASM Builds
//! - All tool control via RPC from React frontend
//! - No keyboard shortcuts (controlled programmatically)
//! - UI rendered in React, communicates via JSON-RPC 2.0
//!
//! ## Tool Coordination Events
//!
//! - `ToolSelectionEvent`: Activate specific tool (keyboard or RPC source)
//! - `ClearToolEvent`: Deactivate current tool and reset state
//! - `PolygonActionEvent`: Complete, clear, or clear all polygons
//! - `AssetPlacementEvent`: Select asset, place at position, toggle mode
//! - `PolygonHideRequestEvent`: Queue hide operation with mask filters
//! - `PolygonReclassifyRequestEvent`: Queue reclassification with target class

/// Point selection tool for connectivity class ID queries (native only).
///
/// Legacy debugging tool for inspecting connectivity data via mouse clicks.
pub mod class_selection;

/// Distance measurement tool with heightmap-aware terrain following.
///
/// Two-point measurement with live preview and RPC distance notifications.
pub mod measure;

/// Polygon-based point reclassification and hiding operations.
///
/// Interactive polygon drawing with compute shader integration for classification changes.
pub mod polygon;

/// Unified tool manager coordinating exclusive tool activation and state.
///
/// Handles tool selection events from keyboard shortcuts and RPC with frontend notifications.
pub mod tool_manager;

/// Asset placement and manipulation tool with instanced rendering.
///
/// OBB-based selection, scroll-wheel rotation, and GPU-efficient instance management.
pub mod asset_manager;
