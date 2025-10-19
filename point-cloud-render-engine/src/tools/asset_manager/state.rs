use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;

// Resources
#[derive(Resource)]
pub struct AssetManagerUiState {
    pub collapsed: bool,
    pub open_width: f32,
    pub closed_width: f32,
}
impl Default for AssetManagerUiState {
    fn default() -> Self {
        Self {
            collapsed: false,
            open_width: 280.0,
            closed_width: 32.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct PlaceAssetBoundState {
    pub active: bool,
    pub selected_asset_name: Option<String>,
}

#[derive(Resource)]
pub struct RotationSettings {
    pub speed: f32,
}
impl Default for RotationSettings {
    fn default() -> Self {
        Self { speed: 0.02 }
    }
}

#[derive(Resource, Clone, Default)]
pub struct PlacedAssetInstances {
    pub instances: Vec<PlacedAssetInstance>,
}

// Components
#[derive(Component)]
pub struct AssetManagerRoot;
#[derive(Component)]
pub struct AssetManagerBody;
#[derive(Component)]
pub struct HeaderNode;
#[derive(Component)]
pub struct TitleText;
#[derive(Component)]
pub struct CollapseButton;
#[derive(Component)]
pub struct CollapseLabel;
#[derive(Component)]
pub struct PlaceCubeButton;
#[derive(Component)]
pub struct PlaceCubeLabel;
#[derive(Component)]
pub struct ClearBoundsButton;
#[derive(Component)]
pub struct PlacedBounds;
#[derive(Component)]
pub struct ActiveRotating;
#[derive(Component)]
pub struct Selected;
#[derive(Component)]
pub struct BoundsSize(pub Vec3);

// Per-placed instance data
#[derive(Component, Clone, ExtractComponent)]
pub struct PlacedAssetInstance {
    pub asset_name: String,
    pub transform: Transform,
    pub uv_bounds: Vec4,
}
