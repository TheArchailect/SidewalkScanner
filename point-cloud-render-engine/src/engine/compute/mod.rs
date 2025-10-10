/// Phase 1 Compute Pipeline: Procedural Texture Reclassification
///
/// This compute pass is the first stage of the rendering pipeline. It procedurally modifies
/// encoded textures (e.g., classification and RGB data) using user-defined polygon masks and
/// spatial filtering, producing a new classification texture used downstream.
///
/// ### Inputs:
/// - `original_texture`: Source RGB + classification (stored in alpha)
/// - `position_texture`: Normalized world positions (xyz) + connectivity class ID (a)
/// - `spatial_index_texture`: Precomputed Morton codes for optional spatial acceleration
/// - `compute_data` (uniform): Contains polygon definitions, mask entries, selection info, etc.
/// - `bounds` (uniform): World-space bounding box used for position denormalization
///
/// ### Outputs:
/// - `output_texture`: A reclassified texture with updated classification values per point
///
/// ### Behavior:
/// - Applies reclassification or hide operations to points inside polygons, using a modifier-stack-like logic
/// - Supports optional spatial filtering via Morton codes or AABBs to accelerate large polygon tests
/// - Ensures hidden points (class `254`) are not reprocessed by later polygons
/// - Honors per-polygon masking rules through encoded ignore-mask IDs
///
/// ### Special Modes:
/// - `render_mode` controls the output format: raw RGB, reclassified view, morton debug, spatial debug, etc.
/// - Hover/selection highlights can override classification color during user interaction for UX feedback
///
/// ### Notes:
/// - This compute pass is *non-destructive*: classification overrides are written to a separate output texture.
/// - Hidden points are indicated via classification value `254`, which downstream passes interpret as "discard".
/// - Must be executed before any shading or rendering passes that rely on classification data.
///
/// ### Performance:
/// - Spatial filtering significantly reduces cost on large point clouds
/// - Morton filtering is more granular but computationally heavier than AABB mode
pub mod compute_classification;
pub mod edl_compute_depth;
