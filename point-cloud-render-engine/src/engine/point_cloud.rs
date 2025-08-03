use bevy::{
    asset::LoadState,
    prelude::*,
    render::{
        texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
        view::NoFrustumCulling,
    },
};

use serde_json::Value;
use std::fs;

use super::{
    grid::{GridCreated, create_ground_grid},
    materials::PointCloudMaterial,
    mesh::create_point_index_mesh,
};

#[derive(Component)]
pub struct PointCloud;

#[derive(Resource)]
pub struct PointCloudAssets {
    pub position_texture: Handle<Image>,
    pub metadata_texture: Handle<Image>,
    pub heightmap_texture: Handle<Image>,
    pub bounds: Option<PointCloudBounds>,
    pub is_loaded: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct PointCloudBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
    pub total_points: usize,
    pub loaded_points: usize,
    pub texture_size: u32,
}

impl PointCloudBounds {
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            ((self.max_x + self.min_x) * 0.5) as f32,
            ((self.max_y + self.min_y) * 0.5) as f32,
            ((self.max_z + self.min_z) * 0.5) as f32,
        )
    }

    pub fn size(&self) -> Vec3 {
        Vec3::new(
            (self.max_x - self.min_x) as f32,
            (self.max_y - self.min_y) as f32,
            (self.max_z - self.min_z) as f32,
        )
    }

    pub fn ground_height(&self) -> f32 {
        self.min_y as f32
    }
}

pub fn check_textures_loaded(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PointCloudMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut grid_created: ResMut<GridCreated>,
) {
    if assets.is_loaded {
        return;
    }

    let pos_loaded = matches!(
        asset_server.get_load_state(&assets.position_texture),
        Some(LoadState::Loaded)
    );
    let meta_loaded = matches!(
        asset_server.get_load_state(&assets.metadata_texture),
        Some(LoadState::Loaded)
    );
    let heightmap_loaded = matches!(
        asset_server.get_load_state(&assets.heightmap_texture),
        Some(LoadState::Loaded)
    );

    if !pos_loaded || !meta_loaded {
        return;
    }

    if let Some(bounds) = &assets.bounds {
        println!("DDS textures loaded! Creating GPU point cloud...");

        if let Some(position_image) = images.get_mut(&assets.position_texture) {
            position_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Nearest,
                ..default()
            });
        }

        if let Some(metadata_image) = images.get_mut(&assets.metadata_texture) {
            metadata_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Nearest,
                ..default()
            });
        }

        let mesh = create_point_index_mesh(bounds.loaded_points);

        let material = PointCloudMaterial {
            position_texture: assets.position_texture.clone(),
            metadata_texture: assets.metadata_texture.clone(),
            params: [
                Vec4::new(
                    bounds.min_x as f32,
                    bounds.min_y as f32,
                    bounds.min_z as f32,
                    bounds.texture_size as f32,
                ),
                Vec4::new(
                    bounds.max_x as f32,
                    bounds.max_y as f32,
                    bounds.max_z as f32,
                    0.0,
                ),
            ],
        };

        commands.spawn((
            MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: materials.add(material),
                transform: Transform::from_translation(Vec3::ZERO),
                visibility: Visibility::Visible,
                ..default()
            },
            PointCloud,
            NoFrustumCulling,
        ));

        println!(
            "GPU point cloud created! {} points ready for rendering (DDS format)",
            bounds.loaded_points
        );

        // Create grid after point cloud is set up
        if !grid_created.created {
            let heightmap_image = if heightmap_loaded {
                images.get(&assets.heightmap_texture)
            } else {
                None
            };

            create_ground_grid(
                &mut commands,
                bounds,
                &mut meshes,
                &mut standard_materials,
                heightmap_image,
            );

            grid_created.created = true;

            if heightmap_loaded {
                println!("Heightfield-aware grid created!");
            } else {
                println!("Flat grid created (heightmap not loaded yet).");
            }
        }

        assets.is_loaded = true;
    }
}

pub fn load_bounds(path: &str) -> Result<PointCloudBounds, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;

    Ok(PointCloudBounds {
        min_x: json["min_x"].as_f64().unwrap(),
        max_x: json["max_x"].as_f64().unwrap(),
        min_y: json["min_y"].as_f64().unwrap(),
        max_y: json["max_y"].as_f64().unwrap(),
        min_z: json["min_z"].as_f64().unwrap(),
        max_z: json["max_z"].as_f64().unwrap(),
        total_points: json["total_points"].as_u64().unwrap() as usize,
        loaded_points: json["loaded_points"].as_u64().unwrap() as usize,
        texture_size: json["texture_size"].as_u64().unwrap() as u32,
    })
}

pub fn sample_heightmap(
    heightmap_image: &Image,
    norm_x: f32,
    norm_z: f32,
    bounds: &PointCloudBounds,
) -> f32 {
    let width = 2048; // Your heightmap size
    let height = 2048;

    let pixel_x = ((norm_x * (width - 1) as f32) as usize).min(width - 1);
    let pixel_y = ((norm_z * (height - 1) as f32) as usize).min(height - 1);

    // Sample R32_Float texture data
    let pixel_index = (pixel_y * width + pixel_x) * 4; // 4 bytes per f32
    let height_bytes = &heightmap_image.data[pixel_index..pixel_index + 4];
    let normalized_height = f32::from_le_bytes([
        height_bytes[0],
        height_bytes[1],
        height_bytes[2],
        height_bytes[3],
    ]);

    // Denormalize height
    bounds.min_y as f32 + normalized_height * (bounds.max_y - bounds.min_y) as f32
}
