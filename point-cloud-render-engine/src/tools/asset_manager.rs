use bevy::prelude::*;

pub struct AssetManagerUiPlugin;

impl Plugin for AssetManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManagerUiState>()
            .add_systems(Startup, spawn_asset_manager_ui)
            .add_systems(
                Update,
                (
                    collapse_button_interaction,
                    keyboard_toggle_collapse,
                    apply_collapse_state, 
                ),
            );
    }
}

// Collapsible panel state
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

#[derive(Component)] struct AssetManagerRoot;
#[derive(Component)] struct AssetManagerBody;
#[derive(Component)] struct HeaderNode;
#[derive(Component)] struct TitleText;
#[derive(Component)] struct CollapseButton;
#[derive(Component)] struct CollapseLabel;

fn spawn_asset_manager_ui(mut commands: Commands, state: Res<AssetManagerUiState>) {
    let width = if state.collapsed { state.closed_width } else { state.open_width };
    let body_display = if state.collapsed { Display::None } else { Display::Flex };

    // Root panel fixed to the right, full height
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
            // Header (title + chevron button)
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
                        justify_content: if state.collapsed {
                            JustifyContent::FlexEnd
                        } else {
                            JustifyContent::SpaceBetween
                        },
                        ..default()
                    },
                ))
                .with_children(|header| {
                    // Title (hidden when collapsed)
                    header.spawn((
                        TitleText,
                        Name::new("Title"),
                        Text::new("Asset Manager"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Node {
                            display: if state.collapsed { Display::None } else { Display::Flex },
                            ..default()
                        },
                    ));

                    // Collapse/expand button (chevron)
                    let chevron = if state.collapsed { "›" } else { "‹" }; // right/left
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
            parent.spawn((
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
            ));
        });
}

// Chevron toggle
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
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.22, 0.24, 0.28));
            }
        }
    }
}

// Prevents B0001 panic, applies collapse state to UI nodes
fn apply_collapse_state(
    state: Res<AssetManagerUiState>,
    
    mut nodes: ParamSet<(
        Query<&mut Node, With<AssetManagerRoot>>,      // p0
        Query<&mut Node, With<AssetManagerBody>>,      // p1
        Query<&mut Node, With<HeaderNode>>,            // p2
        Query<&mut Node, With<TitleText>>,             // p3
        Query<&mut Node, With<CollapseButton>>,        // p4
    )>,
    
    mut chevrons: Query<&mut Text, With<CollapseLabel>>,
) {
    if !state.is_changed() {
        return;
    }

    // Root width
    if let Ok(mut node) = nodes.p0().get_single_mut() {
        node.width = Val::Px(if state.collapsed { state.closed_width } else { state.open_width });
    }

    // Body visibility
    if let Ok(mut node) = nodes.p1().get_single_mut() {
        node.display = if state.collapsed { Display::None } else { Display::Flex };
    }

    // Header padding + alignment
    if let Ok(mut node) = nodes.p2().get_single_mut() {
        let pad = if state.collapsed { 4.0 } else { 12.0 };
        node.padding = UiRect::all(Val::Px(pad));
        node.justify_content = if state.collapsed {
            JustifyContent::FlexEnd
        } else {
            JustifyContent::SpaceBetween
        };
    }

    // Title visibility
    if let Ok(mut node) = nodes.p3().get_single_mut() {
        node.display = if state.collapsed { Display::None } else { Display::Flex };
    }

    // Button size
    if let Ok(mut node) = nodes.p4().get_single_mut() {
        let s = if state.collapsed { 24.0 } else { 28.0 };
        node.width = Val::Px(s);
        node.height = Val::Px(s);
    }

    // Chevron 
    for mut text in &mut chevrons {
        *text = Text::new(if state.collapsed { ">" } else { "<" });
    }
}
