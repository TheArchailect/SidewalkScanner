<<<<<<< HEAD
=======
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::{Mesh3d, MeshMaterial3d};
use bevy::math::primitives::Cuboid;
use bevy::render::mesh::Mesh;
use bevy::render::alpha::AlphaMode;
use bevy::input::mouse::MouseWheel;
use crate::engine::camera::viewport_camera::ViewportCamera;
>>>>>>> ce968a1b1981d3033784c6ca8c06de5ec7cef752
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::point_cloud_render_pipeline::PointCloudRenderable;
use bevy::math::primitives::Cuboid;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::mesh::Mesh;
use bevy::window::PrimaryWindow;

#[derive(Component, Clone, ExtractComponent)]
pub struct PlacedAssetInstance {
    pub asset_name: String,
    pub transform: Transform,
    pub uv_bounds: Vec4,
}

#[derive(Resource, Clone, Default)]
pub struct PlacedAssetInstances {
    pub instances: Vec<PlacedAssetInstance>,
}

pub struct AssetManagerUiPlugin;

impl Plugin for AssetManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManagerUiState>()
            .init_resource::<PlaceCubeState>()
            .insert_resource(RotationSettings::default())
            .insert_resource(SelectionLock::default())
            .add_systems(Startup, spawn_asset_manager_ui)
            .add_systems(
                Update,
                (
                    // UI interactions
                    collapse_button_interaction,
                    apply_collapse_state,
                    place_cube_button_interaction,
                    reflect_place_cube_button,
                    clear_bounds_button_interaction,
                    // Asset selection number keys (1-9)
                    asset_selection_hotkeys,
                    reflect_selected_asset_label,
<<<<<<< HEAD
                    place_cube_on_world_click,
=======

                    place_cube_on_world_click,

                    // Bounds selection/rotation
                    toggle_select_on_click,
                    rotate_active_bounds_on_scroll,
                    reflect_selection_lock,
>>>>>>> ce968a1b1981d3033784c6ca8c06de5ec7cef752
                ),
            );
    }
}

#[derive(Resource)]
struct AssetManagerUiState {
    collapsed: bool,
    open_width: f32,
    closed_width: f32,
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

#[derive(Resource)]
struct PlaceCubeState {
    active: bool,
    cube_size: f32,
    selected_asset_name: Option<String>,
}
impl Default for PlaceCubeState {
    fn default() -> Self {
        Self {
            active: false,
            cube_size: 1.0,
            selected_asset_name: None,
        }
    }
}

<<<<<<< HEAD
#[derive(Component)]
struct AssetManagerRoot;
#[derive(Component)]
struct AssetManagerBody;
#[derive(Component)]
struct HeaderNode;
#[derive(Component)]
struct TitleText;
#[derive(Component)]
struct CollapseButton;
#[derive(Component)]
struct CollapseLabel;
#[derive(Component)]
struct PlaceCubeButton;
#[derive(Component)]
struct PlaceCubeLabel;
#[derive(Component)]
struct ClearBoundsButton;
#[derive(Component)]
struct PlacedBounds;
=======
#[derive(Resource)]
struct RotationSettings {
    speed: f32,
    snap_deg: f32,
}

impl Default for RotationSettings {
    fn default() -> Self {
        Self { speed: 0.18, snap_deg: 15.0 }
    }
}

#[derive(Resource, Default)]
pub struct SelectionLock {
    pub active: bool,
}


#[derive(Component)] struct AssetManagerRoot;
#[derive(Component)] struct AssetManagerBody;
#[derive(Component)] struct HeaderNode;
#[derive(Component)] struct TitleText;
#[derive(Component)] struct CollapseButton;
#[derive(Component)] struct CollapseLabel;
#[derive(Component)] struct PlaceCubeButton;
#[derive(Component)] struct PlaceCubeLabel;
#[derive(Component)] struct ClearBoundsButton;

// Marker components for placed bounds/wireframe entities
#[derive(Component)] pub struct PlacedBounds;    
#[derive(Component)] struct ActiveRotating;        
#[derive(Component)] struct Selected;             
#[derive(Component)] struct BoundsSize(Vec3);       

>>>>>>> ce968a1b1981d3033784c6ca8c06de5ec7cef752

fn spawn_asset_manager_ui(mut commands: Commands, state: Res<AssetManagerUiState>) {
    let width = if state.collapsed {
        state.closed_width
    } else {
        state.open_width
    };
    let body_display = if state.collapsed {
        Display::None
    } else {
        Display::Flex
    };

    commands
        .spawn((
            AssetManagerRoot,
            Name::new("AssetManagerPanel"),
            BackgroundColor(Color::srgb(0.10, 0.11, 0.13)),
            Node {
                width: Val::Px(width),
                min_width: Val::Px(0.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexStart,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|parent| {
            let (pad, btn) = if state.collapsed {
                (4.0, 24.0)
            } else {
                (12.0, 28.0)
            };
            parent
                .spawn((
                    HeaderNode,
                    Name::new("Header"),
                    BackgroundColor(Color::srgb(0.14, 0.16, 0.20)),
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(pad)),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: if state.collapsed {
                            JustifyContent::FlexEnd
                        } else {
                            JustifyContent::SpaceBetween
                        },
                        ..default()
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        TitleText,
                        Name::new("Title"),
                        Text::new("Asset Manager"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Node {
                            display: if state.collapsed {
                                Display::None
                            } else {
                                Display::Flex
                            },
                            ..default()
                        },
                    ));

                    let chevron = if state.collapsed { ">" } else { "<" };
                    header
                        .spawn((
                            CollapseButton,
                            Name::new("CollapseButton"),
                            Button,
                            BackgroundColor(Color::srgb(0.22, 0.24, 0.28)),
                            BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
                            Node {
                                width: Val::Px(btn),
                                height: Val::Px(btn),
                                display: Display::Flex,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                        ))
                        .with_children(|btn_parent| {
                            btn_parent.spawn((
                                CollapseLabel,
                                Text::new(chevron),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                            ));
                        });
                });

            // Body
            parent
                .spawn((
                    AssetManagerBody,
                    Name::new("Body"),
                    BackgroundColor(Color::srgb(0.12, 0.13, 0.15)),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        row_gap: Val::Px(8.0),
                        column_gap: Val::Px(8.0),
                        display: body_display,
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                ))
                .with_children(|body| {
                    // Place Bounds button
                    body.spawn((
                        PlaceCubeButton,
                        Button,
                        Name::new("PlaceCubeButton"),
                        // Default color
                        BackgroundColor(Color::srgb(0.22, 0.24, 0.28)),
                        BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            PlaceCubeLabel,
                            Text::new("Place Bounds (first asset)"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });

                    // Clear All button
                    body.spawn((
                        ClearBoundsButton,
                        Button,
                        Name::new("ClearBoundsButton"),
                        BackgroundColor(Color::srgb(0.28, 0.10, 0.10)),
                        BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Clear All Bounds"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });
                });
        });
}

fn collapse_button_interaction(
    mut q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<CollapseButton>),
    >,
    mut state: ResMut<AssetManagerUiState>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                state.collapsed = !state.collapsed;
                *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.24));
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.22, 0.24, 0.28)),
        }
    }
}

fn apply_collapse_state(
    state: Res<AssetManagerUiState>,
    mut nodes: ParamSet<(
        Query<&mut Node, With<AssetManagerRoot>>,
        Query<&mut Node, With<AssetManagerBody>>,
        Query<&mut Node, With<HeaderNode>>,
        Query<&mut Node, With<TitleText>>,
        Query<&mut Node, With<CollapseButton>>,
    )>,
    mut chevrons: Query<&mut Text, With<CollapseLabel>>,
) {
    if !state.is_changed() {
        return;
    }

    // Width
    if let Ok(mut n) = nodes.p0().single_mut() {
        n.width = Val::Px(if state.collapsed {
            state.closed_width
        } else {
            state.open_width
        });
    }
    // Body visibility
    if let Ok(mut n) = nodes.p1().single_mut() {
        n.display = if state.collapsed {
            Display::None
        } else {
            Display::Flex
        };
    }
    // Header padding/justify
    if let Ok(mut n) = nodes.p2().single_mut() {
        let pad = if state.collapsed { 4.0 } else { 12.0 };
        n.padding = UiRect::all(Val::Px(pad));
        n.justify_content = if state.collapsed {
            JustifyContent::FlexEnd
        } else {
            JustifyContent::SpaceBetween
        };
    }
    // Title visibility
    if let Ok(mut n) = nodes.p3().single_mut() {
        n.display = if state.collapsed {
            Display::None
        } else {
            Display::Flex
        };
    }
    // Button size
    if let Ok(mut n) = nodes.p4().single_mut() {
        let s = if state.collapsed { 24.0 } else { 28.0 };
        n.width = Val::Px(s);
        n.height = Val::Px(s);
    }
    // Chevron (> & <) for UI open close
    for mut t in &mut chevrons {
        *t = Text::new(if state.collapsed { ">" } else { "<" });
    }
}

// TODO: FIX place_cube_button_interaction & reflect_place_cube_button logic for highlight colours

fn place_cube_button_interaction(
    mut q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<PlaceCubeButton>),
    >,
    mut place: ResMut<PlaceCubeState>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                place.active = !place.active;
                *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.24));
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32)),
            Interaction::None => {
                *bg = BackgroundColor(if place.active {
                    Color::srgb(0.0, 0.90, 0.0)
                } else {
                    Color::srgb(0.22, 0.24, 0.28)
                })
            }
        }
    }
}

fn reflect_place_cube_button(
    place: Res<PlaceCubeState>,
    mut q: Query<&mut BackgroundColor, With<PlaceCubeButton>>,
) {
    if !place.is_changed() {
        return;
    }
    if let Ok(mut bg) = q.single_mut() {
        *bg = BackgroundColor(if place.active {
            Color::srgb(0.30, 0.34, 0.40)
        } else {
            Color::srgb(0.22, 0.24, 0.28)
        });
    }
}

// Clears all placed wireframe/bounds entities
fn clear_bounds_button_interaction(
    mut q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<ClearBoundsButton>),
    >,
    mut commands: Commands,
    to_clear: Query<Entity, With<PlacedBounds>>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                for e in &to_clear {
                    commands.entity(e).despawn();
                }
                *bg = BackgroundColor(Color::srgb(0.20, 0.12, 0.12));
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.34, 0.14, 0.14)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.28, 0.10, 0.10)),
        }
    }
}

// Asset selection hotkeys (1-9) to change selected asset for placement
fn asset_selection_hotkeys(
    kb: Res<ButtonInput<KeyCode>>,
    manifests: Res<Assets<SceneManifest>>,
    assets: Res<PointCloudAssets>,
    mut place: ResMut<PlaceCubeState>,
) {
    let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) else {
        return;
    };
    let Some(aa) = manifest.asset_atlas.as_ref() else {
        return;
    };
    if aa.assets.is_empty() {
        return;
    }

    let mut idx = place
        .selected_asset_name
        .as_ref()
        .and_then(|name| aa.assets.iter().position(|a| &a.name == name))
        .unwrap_or(0);

    let mut changed = false;

    for n in 1..=9 {
        let key = match n {
            1 => KeyCode::Digit1,
            2 => KeyCode::Digit2,
            3 => KeyCode::Digit3,
            4 => KeyCode::Digit4,
            5 => KeyCode::Digit5,
            6 => KeyCode::Digit6,
            7 => KeyCode::Digit7,
            8 => KeyCode::Digit8,
            _ => KeyCode::Digit9,
        };
        if kb.just_pressed(key) {
            if (n as usize) <= aa.assets.len() {
                idx = n as usize - 1;
                changed = true;
            }
            break;
        }
    }

    if changed {
        let new_name = aa.assets[idx].name.clone();
        place.selected_asset_name = Some(new_name.clone());
        info!("Selected asset: {}", new_name);
    }
}

// Change selected asset labels in UI
fn reflect_selected_asset_label(
    place: Res<PlaceCubeState>,
    manifests: Res<Assets<SceneManifest>>,
    assets: Res<PointCloudAssets>,
    mut q: Query<&mut Text, With<PlaceCubeLabel>>,
) {
    if q.is_empty() {
        return;
    }

    let label = if let Some(name) = place.selected_asset_name.as_ref() {
        format!("Place Bounds ({})", name)
    } else {
        if let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) {
            if let Some(first) = manifest
                .asset_atlas
                .as_ref()
                .and_then(|aa| aa.assets.first())
            {
                format!("Place Bounds ({})", first.name)
            } else {
                "Place Bounds".to_string()
            }
        } else {
            "Place Bounds".to_string()
        }
    };

    if let Ok(mut t) = q.single_mut() {
        if t.0 != label {
            *t = Text::new(label);
        }
    }
}

// Spawn wireframe cuboids, assets loaded from manifest
fn place_cube_on_world_click(
    buttons: Res<ButtonInput<MouseButton>>,
    place: Res<PlaceCubeState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut maps_camera: Option<ResMut<ViewportCamera>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !place.active || !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(mut maps_camera) = maps_camera else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((cam_xform, camera)) = cameras.single() else {
        return;
    };
    let Some(scene_bounds) = assets.get_bounds(&manifests) else {
        return;
    };
    let Some(height_img) = images.get(&assets.heightmap_texture) else {
        return;
    };

    // Raycast
    let hit = maps_camera.mouse_to_ground_plane(
        cursor_pos,
        camera,
        cam_xform,
        Some(height_img),
        &scene_bounds,
    );
    let Some(hit) = hit else {
        return;
    };

    let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) else {
        return;
    };

    // Gets asset currently selected or defaults to first asset
    let picked = if let Some(ref name) = place.selected_asset_name {
        manifest
            .asset_atlas
            .as_ref()
            .and_then(|aa| aa.assets.iter().find(|a| a.name == *name))
    } else {
        manifest
            .asset_atlas
            .as_ref()
            .and_then(|aa| aa.assets.first())
    };
    let Some(asset_meta) = picked else {
        return;
    };

    // Locals bounds to determine size
    let lb = &asset_meta.local_bounds;
    let mut sx = (lb.max_x - lb.min_x) as f32;
    let mut sy = (lb.max_y - lb.min_y) as f32;
    let mut sz = (lb.max_z - lb.min_z) as f32;
    if !sx.is_finite() || !sy.is_finite() || !sz.is_finite() {
        return;
    }
    if sx <= 0.0 {
        sx = 0.001;
    }
    if sy <= 0.0 {
        sy = 0.001;
    }
    if sz <= 0.0 {
        sz = 0.001;
    }
    let size = Vec3::new(sx, sy, sz);

    // Uses center of bounds for placement
    // Adjusts Y to sit on ground plane by half height
    let center = Vec3::new(hit.x, hit.y + size.y * 0.5, hit.z);
    let transform = Transform::from_translation(center);

    let uv_bounds = Vec4::new(
        asset_meta.uv_bounds.uv_min[0],
        asset_meta.uv_bounds.uv_min[1],
        asset_meta.uv_bounds.uv_max[0],
        asset_meta.uv_bounds.uv_max[1],
    );

    let mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        alpha_mode: AlphaMode::Blend, // change for transparency
        unlit: true,
        emissive: Color::srgba(0.0, 0.0, 0.0, 0.0).into(),
        perceptual_roughness: 1.0,
        ..default()
    });

    // Spawn wireframe
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(size))),
        MeshMaterial3d(mat),
        Transform::from_translation(center),
        Wireframe,
        WireframeColor {
            color: Color::WHITE,
        },
        PlacedAssetInstance {
            asset_name: asset_meta.name.clone(),
            transform,
            uv_bounds,
        },
        // Mark for point cloud rendering
        PointCloudRenderable {
            point_count: asset_meta.point_count as u32,
        },
        PlacedBounds,
        BoundsSize(size),
        Name::new(format!("{}_bounds_wire", asset_meta.name)),
    ));

    info!("Placed bounds for '{}' at {:?}", asset_meta.name, center);
}
<<<<<<< HEAD
=======

// Bounds/wireframe selection system
// Click to select/deselect, only one selected at a time
fn toggle_select_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    q_bounds: Query<(Entity, &GlobalTransform, &BoundsSize, Option<&Selected>), With<PlacedBounds>>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }

    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((cam_xf, camera)) = cameras.get_single() else { return; };

    let Ok(ray) = camera.viewport_to_world(cam_xf, cursor_pos) else { return; };
    let origin = ray.origin;
    let dir = ray.direction.as_vec3();;

    // Find nearest hit among bounds
    let mut best: Option<(Entity, f32, bool)> = None; // (entity, distance t, was_selected)

    for (e, xf, BoundsSize(size), selected) in &q_bounds {
        if let Some(t) = ray_hits_obb(origin, dir, *xf, *size) {
            if t > 0.0 && (best.is_none() || t < best.unwrap().1) {
                best = Some((e, t, selected.is_some()));
            }
        }
    }

    // Toggle selection
    if let Some((hit_e, _t, was_selected)) = best {
        // Deselect all
        for (e, _, _, sel) in &q_bounds {
            if sel.is_some() {
                commands.entity(e).remove::<Selected>();
                commands.entity(e).remove::<ActiveRotating>();
                commands.entity(e).insert(WireframeColor { color: Color::WHITE });
            }
        }

        // Deselect 
        if !was_selected {
            commands.entity(hit_e).insert(Selected);
            commands.entity(hit_e).insert(ActiveRotating);
            commands.entity(hit_e).insert(WireframeColor { color: Color::srgb(1.0, 1.0, 0.0) });
        }
    }
}

// Rotate selected bounds on mouse wheel scroll
fn rotate_active_bounds_on_scroll(
    mut wheel: EventReader<MouseWheel>,
    mut q: Query<&mut Transform, (With<ActiveRotating>, With<Selected>)>,
    settings: Res<RotationSettings>,
) {
    if q.is_empty() { return; }

    let mut delta = 0.0f32;
    for ev in wheel.read() {
        delta += ev.y as f32;
    }
    if delta.abs() < f32::EPSILON { return; }

    let mut angle = delta * settings.speed;

    for mut t in &mut q {
        t.rotate(Quat::from_axis_angle(Vec3::Y, angle));
    }
}

fn reflect_selection_lock(
    q_selected: Query<(), With<Selected>>,
    mut lock: ResMut<SelectionLock>,
) {
    lock.active = !q_selected.is_empty();
}


fn ray_hits_obb(origin: Vec3, dir: Vec3, xf: GlobalTransform, size: Vec3) -> Option<f32> {
    // Inverts box transform
    let inv = xf.compute_matrix().inverse();
    // Transforms ray into local space of box
    let o_local = inv.transform_point3(origin);
    let d_local = inv.transform_vector3(dir);
    // Half extents
    let he = size * 0.5;
    ray_aabb_hit_t(o_local, d_local, -he, he)
}

// Slab method for ray-AABB intersection
// Returns Some t (how far along the ray), where ray hits AABB, none if no hit
fn ray_aabb_hit_t(ray_origin: Vec3, ray_direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {

    // Inverse direction
    let inv = Vec3::new(
        if ray_direction.x != 0.0 { 1.0 / ray_direction.x } else { f32::INFINITY },
        if ray_direction.y != 0.0 { 1.0 / ray_direction.y } else { f32::INFINITY },
        if ray_direction.z != 0.0 { 1.0 / ray_direction.z } else { f32::INFINITY },
    );

    // X slab intersection
    let mut tmin = (min.x - ray_origin.x) * inv.x;
    let mut tmax = (max.x - ray_origin.x) * inv.x;
    if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

    // Y slab intersection
    let mut tymin = (min.y - ray_origin.y) * inv.y;
    let mut tymax = (max.y - ray_origin.y) * inv.y;
    if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

    if (tmin > tymax) || (tymin > tmax) { return None; }
    if tymin > tmin { tmin = tymin; }
    if tymax < tmax { tmax = tymax; }

    // Z slab intersection
    let mut tzmin = (min.z - ray_origin.z) * inv.z;
    let mut tzmax = (max.z - ray_origin.z) * inv.z;
    if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

    if (tmin > tzmax) || (tzmin > tmax) { return None; }
    if tzmin > tmin { tmin = tzmin; }
    if tzmax < tmax { tmax = tzmax; }

    // Check if behind ray
    if tmax < 0.0 { return None; }   
    Some(if tmin >= 0.0 { tmin } else { tmax })
}
>>>>>>> ce968a1b1981d3033784c6ca8c06de5ec7cef752
