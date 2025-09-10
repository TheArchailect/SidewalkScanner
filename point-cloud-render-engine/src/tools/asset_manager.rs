use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;

pub struct AssetManagerUiPlugin;

//TODO: 
// - load geometry instead of a cube, as a bound. Check already loaded Manifest json.
// - Colour helpers for UI states
// - Fix frame lag when clicking button and changing colour

// Plugin setup for asset manager UI panel, view app_setup.rs
impl Plugin for AssetManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManagerUiState>()
            .init_resource::<PlaceCubeState>() 
            .add_systems(Startup, spawn_asset_manager_ui)
            .add_systems(
                Update,
                (
                    // UI interactions
                    collapse_button_interaction,
                    apply_collapse_state,         

                    place_cube_button_interaction, 
                    reflect_place_cube_button,    

                    place_cube_on_world_click, 
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
        Self { collapsed: false, open_width: 280.0, closed_width: 32.0 } 
    }
}

#[derive(Resource)]
struct PlaceCubeState {
    active: bool,
    cube_size: f32,
}
impl Default for PlaceCubeState {
    fn default() -> Self {
        Self { active: false, cube_size: 1.0 }
    }
}

#[derive(Component)] struct AssetManagerRoot;
#[derive(Component)] struct AssetManagerBody;
#[derive(Component)] struct HeaderNode;
#[derive(Component)] struct TitleText;
#[derive(Component)] struct CollapseButton;
#[derive(Component)] struct CollapseLabel;
#[derive(Component)] struct PlaceCubeButton;

fn spawn_asset_manager_ui(mut commands: Commands, state: Res<AssetManagerUiState>) {
    let width = if state.collapsed { state.closed_width } else { state.open_width };
    let body_display = if state.collapsed { Display::None } else { Display::Flex };

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
           
            let (pad, btn) = if state.collapsed { (4.0, 24.0) } else { (12.0, 28.0) };
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
                        justify_content: if state.collapsed { JustifyContent::FlexEnd } else { JustifyContent::SpaceBetween },
                        ..default()
                    },
                ))
                .with_children(|header| {
                    header.spawn((
                        TitleText,
                        Name::new("Title"),
                        Text::new("Asset Manager"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Node { display: if state.collapsed { Display::None } else { Display::Flex }, ..default() },
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
                                TextFont { font_size: 18.0, ..default() },
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
                            Text::new("Place Cube"),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });
                });
        });
}

fn collapse_button_interaction(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>, With<CollapseButton>)>,
    mut state: ResMut<AssetManagerUiState>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                state.collapsed = !state.collapsed;
                *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.24));
            }
            Interaction::Hovered =>  *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32)),
            Interaction::None =>     *bg = BackgroundColor(Color::srgb(0.22, 0.24, 0.28)),
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
    if !state.is_changed() { return; }

    // Width
    if let Ok(mut n) = nodes.p0().get_single_mut() {
        n.width = Val::Px(if state.collapsed { state.closed_width } else { state.open_width });
    }
    // Body visibility
    if let Ok(mut n) = nodes.p1().get_single_mut() {
        n.display = if state.collapsed { Display::None } else { Display::Flex };
    }
    // Header padding/justify
    if let Ok(mut n) = nodes.p2().get_single_mut() {
        let pad = if state.collapsed { 4.0 } else { 12.0 };
        n.padding = UiRect::all(Val::Px(pad));
        n.justify_content = if state.collapsed { JustifyContent::FlexEnd } else { JustifyContent::SpaceBetween };
    }
    // Title visibility
    if let Ok(mut n) = nodes.p3().get_single_mut() {
        n.display = if state.collapsed { Display::None } else { Display::Flex };
    }
    // Button size
    if let Ok(mut n) = nodes.p4().get_single_mut() {
        let s = if state.collapsed { 24.0 } else { 28.0 };
        n.width = Val::Px(s);
        n.height = Val::Px(s);
    }
    // Glyph
    for mut t in &mut chevrons {
        *t = Text::new(if state.collapsed { ">" } else { "<" });
    }
}

// Toggle tool on button click
fn place_cube_button_interaction(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>, With<PlaceCubeButton>)>,
    mut place: ResMut<PlaceCubeState>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                place.active = !place.active;
                *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.24)); 
            }
            Interaction::Hovered =>  *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32)),
            Interaction::None =>     *bg = BackgroundColor(if place.active { Color::srgb(0.0, 0.90, 0.0) } else { Color::srgb(0.22, 0.24, 0.28) }),
        }
    }
}

// Visually reflect active/inactive state
fn reflect_place_cube_button(
    place: Res<PlaceCubeState>,
    mut q: Query<&mut BackgroundColor, With<PlaceCubeButton>>,
) {
    if !place.is_changed() { return; }
    if let Ok(mut bg) = q.get_single_mut() {
        *bg = BackgroundColor(if place.active { Color::srgb(0.30, 0.34, 0.40) } else { Color::srgb(0.22, 0.24, 0.28) });
    }
}

// Cursor click place cube at ground plane under cursor
fn place_cube_on_world_click(
    buttons: Res<ButtonInput<MouseButton>>,
    place: Res<PlaceCubeState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut maps_camera: ResMut<ViewportCamera>,
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

    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((cam_xform, camera)) = cameras.get_single() else { return; };


    let Some(bounds) = assets.get_bounds(&manifests) else { return; };
    let hit = maps_camera.mouse_to_ground_plane(
        cursor_pos,
        camera,
        cam_xform,
        images.get(&assets.heightmap_texture),
        &bounds,
    );
    let Some(hit) = hit else { return; };

    // Spawn the cube at the intersection
    let s = place.cube_size.max(0.01);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(s, s, s))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.9),
            ..default()
        })),
        Transform::from_xyz(hit.x, hit.y + s * 0.5, hit.z),
    ));
}


