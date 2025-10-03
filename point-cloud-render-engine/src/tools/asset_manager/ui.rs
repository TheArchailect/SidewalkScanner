use bevy::prelude::*;
use super::state::*;

// Spawns the Asset Manager UI panel with header and buttons
pub fn spawn_asset_manager_ui(mut commands: Commands, state: Res<AssetManagerUiState>) {
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
                    // Place Bounds
                    body.spawn((
                        PlaceCubeButton,
                        Button,
                        Name::new("PlaceCubeButton"),
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
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });

                    // Clear All
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
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });
                });
        });
}

pub fn apply_collapse_state(
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

    if let Ok(mut n) = nodes.p0().single_mut() {
        n.width = Val::Px(if state.collapsed { state.closed_width } else { state.open_width });
    }
    if let Ok(mut n) = nodes.p1().single_mut() {
        n.display = if state.collapsed { Display::None } else { Display::Flex };
    }
    if let Ok(mut n) = nodes.p2().single_mut() {
        let pad = if state.collapsed { 4.0 } else { 12.0 };
        n.padding = UiRect::all(Val::Px(pad));
        n.justify_content = if state.collapsed { JustifyContent::FlexEnd } else { JustifyContent::SpaceBetween };
    }
    if let Ok(mut n) = nodes.p3().single_mut() {
        n.display = if state.collapsed { Display::None } else { Display::Flex };
    }
    if let Ok(mut n) = nodes.p4().single_mut() {
        let s = if state.collapsed { 24.0 } else { 28.0 };
        n.width = Val::Px(s);
        n.height = Val::Px(s);
    }
    for mut t in &mut chevrons {
        *t = Text::new(if state.collapsed { ">" } else { "<" });
    }
}

pub fn reflect_place_cube_button(
    place: Res<PlaceAssetBoundState>,
    mut q: Query<&mut BackgroundColor, With<PlaceCubeButton>>,
) {
    if !place.is_changed() { return; }
    if let Ok(mut bg) = q.single_mut() {
        *bg = BackgroundColor(if place.active { Color::srgb(0.30, 0.34, 0.40) } else { Color::srgb(0.22, 0.24, 0.28) });
    }
}

pub fn reflect_selected_asset_label(
    place: Res<PlaceAssetBoundState>,
    manifests: Res<Assets<crate::engine::assets::scene_manifest::SceneManifest>>,
    assets: Res<crate::engine::assets::point_cloud_assets::PointCloudAssets>,
    mut q: Query<&mut Text, With<PlaceCubeLabel>>,
) {
    if q.is_empty() { return; }

    let label = if let Some(name) = place.selected_asset_name.as_ref() {
        format!("Place Bounds ({})", name)
    } else {
        if let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) {
            if let Some(first) = manifest.asset_atlas.as_ref().and_then(|aa| aa.assets.first()) {
                format!("Place Bounds ({})", first.name)
            } else { "Place Bounds".to_string() }
        } else { "Place Bounds".to_string() }
    };

    if let Ok(mut t) = q.single_mut() {
        if t.0 != label { *t = Text::new(label); }
    }
}
