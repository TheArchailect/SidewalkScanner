use bevy::prelude::*;

pub struct AssetManagerUiPlugin;

impl Plugin for AssetManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_asset_manager_ui);
    }
}

#[derive(Component)]
struct AssetManagerRoot;

fn spawn_asset_manager_ui(mut commands: Commands) {

    // Root node for the asset manager panel
    commands
        .spawn((
            Node {
                width: Val::Px(280.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),

                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.11, 0.13)),
            AssetManagerRoot,
            Name::new("AssetManagerPanel"),
        ))
        .with_children(|parent| {
           
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        display: Display::Flex,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.14, 0.16, 0.20)),
                    Name::new("Header"),
                ))
                .with_children(|h| {
                    h.spawn((
                        Text::new("Asset Manager"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Name::new("Title"),
                    ));
                });

            // Placeholder for body content
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    row_gap: Val::Px(8.0),
                    column_gap: Val::Px(8.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.13, 0.15)),
                Name::new("Body"),
            ));
        });
}
