use bevy::prelude::*;
mod engine;
mod tools;

use engine::{
    camera::{MapsCamera, maps_camera_controller},
    gizmos::{update_direction_gizmo, update_mouse_intersection_gizmo},
    grid::GridCreated,
    materials::PointCloudMaterial,
    point_cloud::PointCloudAssets,
    point_cloud::check_textures_loaded,
};

use tools::polygon::{
    PolygonCounter, PolygonTool, polygon_tool_system, update_polygon_preview, update_polygon_render,
};

const ABSOLUTE_ASSET_PATH: &'static str =
    "/home/archailect/Git/SidewalkScanner/point-cloud-render-engine/assets/encoded_textures/warsaw";
const RELATIVE_ASSET_PATH: &'static str = "encoded_textures/warsaw";
const TEXTURE_RESOLUTION: &'static str = "1k";

fn main() {
    let bounds_path = format!("{}_bounds_{}.json", ABSOLUTE_ASSET_PATH, TEXTURE_RESOLUTION);

    let bounds = match engine::point_cloud::load_bounds(&bounds_path) {
        Ok(b) => {
            println!(
                "Loaded bounds: {} points in {}x{} texture",
                b.loaded_points, b.texture_size, b.texture_size
            );
            Some(b)
        }
        Err(e) => {
            eprintln!("Failed to load bounds: {}", e);
            None
        }
    };

    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<PointCloudMaterial>::default(),
        ))
        .insert_resource(PointCloudAssets {
            position_texture: Handle::default(),
            metadata_texture: Handle::default(),
            heightmap_texture: Handle::default(),
            bounds,
            is_loaded: false,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                check_textures_loaded,
                maps_camera_controller,
                display_info,
                update_mouse_intersection_gizmo,
                update_direction_gizmo,
                polygon_tool_system,
                update_polygon_preview,
                update_polygon_render,
            ),
        )
        .init_resource::<PolygonCounter>()
        .init_resource::<PolygonTool>()
        .init_resource::<GridCreated>()
        .run();

    // wasm_bindgen_futures::spawn_local(async {

    // });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
) {
    println!("=== GPU-ACCELERATED POINT CLOUD RENDERER (DDS) ===");

    let position_texture_path = format!(
        "{}_positions_{}.dds",
        ABSOLUTE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let metadata_texture_path = format!(
        "{}_metadata_{}.dds",
        ABSOLUTE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let heightmap_texture_path = format!(
        "{}_road_heightmap_{}.dds",
        ABSOLUTE_ASSET_PATH, TEXTURE_RESOLUTION
    );

    println!("Loading DDS textures for fast GPU processing:");
    println!(
        "  Position: {} (16-bit float, BC6H-ready)",
        position_texture_path
    );
    println!("  Metadata: {} (32-bit float)", metadata_texture_path);

    assets.position_texture = asset_server.load(&position_texture_path);
    assets.metadata_texture = asset_server.load(&metadata_texture_path);
    assets.heightmap_texture = asset_server.load(&heightmap_texture_path);

    // Setup camera
    if let Some(ref bounds) = assets.bounds {
        let maps_camera = MapsCamera::with_bounds(bounds);
        let camera_transform = maps_camera.update_transform();

        commands.spawn(Camera3dBundle {
            transform: camera_transform,
            projection: PerspectiveProjection {
                near: 0.01,
                far: 50000.0,
                fov: 60.0_f32.to_radians(),
                ..default()
            }
            .into(),
            ..default()
        });

        commands.insert_resource(maps_camera);

        // Create axis gizmo at world origin
        commands.spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 1.0,
                height: 10.0,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::GREEN,
                emissive: Color::GREEN * 0.3,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                bounds.ground_height() + 5.0,
                0.0,
            )),
            ..default()
        },));
        commands.spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 1.0,
                height: 10.0,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                emissive: Color::BLUE * 0.3,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                bounds.ground_height() + 5.0,
                0.0,
            ))
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },));

        engine::gizmos::create_direction_gizmo(
            &mut commands,
            bounds,
            &mut meshes,
            &mut materials,
            &asset_server,
        );
    } else {
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });
        commands.insert_resource(MapsCamera::default());
    }

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
        ..default()
    });

    println!("Setup complete. Maps-style camera initialized.");
}

fn display_info(
    keyboard: Res<Input<KeyCode>>,
    assets: Res<PointCloudAssets>,
    maps_camera: Res<MapsCamera>,
) {
    if keyboard.just_pressed(KeyCode::I) {
        if let Some(ref bounds) = assets.bounds {
            println!("=== GPU POINT CLOUD INFO (DDS) ===");
            println!("Points: {} (GPU processed)", bounds.loaded_points);
            println!(
                "Position: {}x{} DDS (16-bit float)",
                bounds.texture_size, bounds.texture_size
            );
            println!(
                "Metadata: {}x{} DDS (32-bit float)",
                bounds.texture_size, bounds.texture_size
            );
            println!(
                "Bounds: ({:.2}, {:.2}, {:.2}) to ({:.2}, {:.2}, {:.2})",
                bounds.min_x, bounds.min_y, bounds.min_z, bounds.max_x, bounds.max_y, bounds.max_z
            );
            println!("=== CAMERA INFO ===");
            println!(
                "Focus: ({:.2}, {:.2}, {:.2})",
                maps_camera.focus_point.x, maps_camera.focus_point.y, maps_camera.focus_point.z
            );
            println!("Height: {:.2}", maps_camera.height);
            println!(
                "Pitch: {:.2}° (looking down)",
                maps_camera.pitch.to_degrees()
            );
            println!("Yaw: {:.2}°", maps_camera.yaw.to_degrees());
        }
    }
}
