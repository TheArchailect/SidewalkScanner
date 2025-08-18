// use crate::engine::shaders::PolygonClassificationUniform;

// use super::{
//     grid::{GridCreated, create_ground_grid},
//     shaders::PointCloudShader,
// };

// use bevy::prelude::*;
// use bevy::{
//     asset::LoadState,
//     image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
//     render::view::NoFrustumCulling,
// };
// use bevy::{render::mesh::PrimitiveTopology, render::render_asset::RenderAssetUsages};

// const HEIGHT_MAP_SIZE: usize = 1024;

// #[derive(Component)]
// pub struct PointCloud;

// #[derive(Resource)]
// pub struct PointCloudAssets {
//     pub position_texture: Handle<Image>,
//     pub metadata_texture: Handle<Image>,
//     pub heightmap_texture: Handle<Image>,
//     pub bounds: Option<PointCloudBounds>,
//     pub is_loaded: bool,
// }

// use serde::{Deserialize, Serialize};

// #[derive(Resource, Debug, Clone, Serialize, Deserialize, bevy::asset::Asset, TypePath)]
// pub struct PointCloudBounds {
//     pub min_x: f64,
//     pub max_x: f64,
//     pub min_y: f64,
//     pub max_y: f64,
//     pub min_z: f64,
//     pub max_z: f64,
//     pub total_points: usize,
//     pub loaded_points: usize,
//     pub texture_size: u32,
//     #[serde(default)]
//     pub sampling_ratio: f64,
//     #[serde(default)]
//     pub utilization_percent: f64,
// }

// impl PointCloudBounds {
//     pub fn center(&self) -> Vec3 {
//         Vec3::new(
//             ((self.max_x + self.min_x) * 0.5) as f32,
//             ((self.max_y + self.min_y) * 0.5) as f32,
//             ((self.max_z + self.min_z) * 0.5) as f32,
//         )
//     }

//     pub fn size(&self) -> Vec3 {
//         Vec3::new(
//             (self.max_x - self.min_x) as f32,
//             (self.max_y - self.min_y) as f32,
//             (self.max_z - self.min_z) as f32,
//         )
//     }

//     pub fn ground_height(&self) -> f32 {
//         self.min_y as f32
//     }
// }

// pub fn check_textures_loaded(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<PointCloudShader>>,
//     mut standard_materials: ResMut<Assets<StandardMaterial>>,
//     mut assets: ResMut<PointCloudAssets>,
//     asset_server: Res<AssetServer>,
//     mut images: ResMut<Assets<Image>>,
//     mut grid_created: ResMut<GridCreated>,
// ) {
//     if assets.is_loaded {
//         return;
//     }

//     let pos_loaded = matches!(
//         asset_server.get_load_state(&assets.position_texture),
//         Some(LoadState::Loaded)
//     );
//     let meta_loaded = matches!(
//         asset_server.get_load_state(&assets.metadata_texture),
//         Some(LoadState::Loaded)
//     );
//     let heightmap_loaded = matches!(
//         asset_server.get_load_state(&assets.heightmap_texture),
//         Some(LoadState::Loaded)
//     );

//     if !pos_loaded || !meta_loaded {
//         return;
//     }

//     if let Some(bounds) = &assets.bounds {
//         println!("DDS textures loaded! Creating GPU point cloud...");

//         if let Some(position_image) = images.get_mut(&assets.position_texture) {
//             position_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
//                 mag_filter: ImageFilterMode::Nearest,
//                 min_filter: ImageFilterMode::Nearest,
//                 ..default()
//             });
//         }

//         if let Some(metadata_image) = images.get_mut(&assets.metadata_texture) {
//             metadata_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
//                 mag_filter: ImageFilterMode::Nearest,
//                 min_filter: ImageFilterMode::Nearest,
//                 ..default()
//             });
//         }

//         let mesh = create_point_index_mesh(bounds.loaded_points);

//         let material = PointCloudShader {
//             position_texture: assets.position_texture.clone(),
//             metadata_texture: assets.metadata_texture.clone(),
//             params: [
//                 Vec4::new(
//                     bounds.min_x as f32,
//                     bounds.min_y as f32,
//                     bounds.min_z as f32,
//                     bounds.texture_size as f32,
//                 ),
//                 Vec4::new(
//                     bounds.max_x as f32,
//                     bounds.max_y as f32,
//                     bounds.max_z as f32,
//                     0.0,
//                 ),
//             ],
//             polygon_data: PolygonClassificationUniform::default(),
//         };

//         commands.spawn((
//             Mesh3d(meshes.add(mesh)),
//             MeshMaterial3d(materials.add(material)),
//             Transform::from_translation(Vec3::ZERO),
//             Visibility::Visible,
//             InheritedVisibility::VISIBLE,
//             ViewVisibility::default(),
//             GlobalTransform::default(),
//             PointCloud,
//             NoFrustumCulling,
//         ));

//         println!(
//             "Point cloud entity spawned with {} vertices",
//             bounds.loaded_points
//         );
//         println!(
//             "Material bounds: min=({:.2}, {:.2}, {:.2}), max=({:.2}, {:.2}, {:.2})",
//             bounds.min_x, bounds.min_y, bounds.min_z, bounds.max_x, bounds.max_y, bounds.max_z
//         );
//         println!("Texture size: {}", bounds.texture_size);

//         println!(
//             "GPU point cloud created! {} points ready for rendering (DDS format)",
//             bounds.loaded_points
//         );

//         // Create grid after point cloud is set up
//         if !grid_created.created {
//             let heightmap_image = if heightmap_loaded {
//                 images.get(&assets.heightmap_texture)
//             } else {
//                 None
//             };

//             create_ground_grid(
//                 &mut commands,
//                 bounds,
//                 &mut meshes,
//                 &mut standard_materials,
//                 heightmap_image,
//             );

//             grid_created.created = true;

//             if heightmap_loaded {
//                 println!("Heightfield-aware grid created!");
//             } else {
//                 println!("Flat grid created (heightmap not loaded yet).");
//             }
//         }

//         assets.is_loaded = true;
//     }
// }

// pub fn create_point_index_mesh(point_count: usize) -> Mesh {
//     let mut mesh = Mesh::new(
//         PrimitiveTopology::PointList,
//         RenderAssetUsages::RENDER_WORLD,
//     );
//     let indices: Vec<[f32; 3]> = (0..point_count).map(|i| [i as f32, 0.0, 0.0]).collect();
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indices);
//     mesh
// }

// pub fn sample_heightmap(
//     heightmap_image: &Image,
//     norm_x: f32,
//     norm_z: f32,
//     bounds: &PointCloudBounds,
// ) -> f32 {
//     let pixel_x = ((norm_x * (HEIGHT_MAP_SIZE - 1) as f32) as usize).min(HEIGHT_MAP_SIZE - 1);
//     let pixel_y = ((norm_z * (HEIGHT_MAP_SIZE - 1) as f32) as usize).min(HEIGHT_MAP_SIZE - 1);

//     // Handle the Option<Vec<u8>> for image data
//     let data = heightmap_image
//         .data
//         .as_ref()
//         .expect("Image data not available");

//     // Sample R32_Float texture data
//     let pixel_index = (pixel_y * HEIGHT_MAP_SIZE + pixel_x) * 4; // 4 bytes per f32
//     let height_bytes = &data[pixel_index..pixel_index + 4];
//     let normalized_height = f32::from_le_bytes([
//         height_bytes[0],
//         height_bytes[1],
//         height_bytes[2],
//         height_bytes[3],
//     ]);

//     // println!(
//     //     "Sampled Height: {}",
//     //     bounds.min_y as f32 + normalized_height * (bounds.max_y - bounds.min_y) as f32
//     // );

//     // Denormalize height
//     bounds.min_y as f32 + normalized_height * (bounds.max_y - bounds.min_y) as f32
// }

/// Point cloud rendering with unified texture pipeline
use crate::engine::shaders::PolygonClassificationUniform;

use super::{
    grid::{GridCreated, create_ground_grid},
    shaders::PointCloudShader,
};

use bevy::prelude::*;
use bevy::{
    asset::LoadState,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    render::view::NoFrustumCulling,
};
use bevy::{render::mesh::PrimitiveTopology, render::render_asset::RenderAssetUsages};

// Use unified texture size constant
const TEXTURE_SIZE: usize = 2048;

#[derive(Component)]
pub struct PointCloud;

/// Point cloud assets using unified texture format
#[derive(Resource)]
pub struct PointCloudAssets {
    pub position_texture: Handle<Image>, // RGBA32F: XYZ + validity
    pub colour_class_texture: Handle<Image>, // RGBA32F: RGB + classification
    pub heightmap_texture: Handle<Image>, // R32F: elevation
    pub bounds: Option<PointCloudBounds>,
    pub is_loaded: bool,
}

use serde::{Deserialize, Serialize};

/// Point cloud bounds and metadata from unified texture pipeline
#[derive(Resource, Debug, Clone, Serialize, Deserialize, bevy::asset::Asset, TypePath)]
pub struct PointCloudBounds {
    pub bounds: BoundsData,
    pub total_points: usize,
    pub loaded_points: usize,
    pub texture_size: u32,
    #[serde(default)]
    pub sampling_ratio: f64,
    #[serde(default = "default_utilisation")]
    pub utilisation_percent: f64,
    #[serde(default)]
    pub has_colour: bool,
    #[serde(default)]
    pub colour_points: usize,
    #[serde(default)]
    pub road_points: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsData {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

fn default_utilisation() -> f64 {
    0.0
}

impl PointCloudBounds {
    /// Calculate centre point of bounds
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            ((self.bounds.max_x + self.bounds.min_x) * 0.5) as f32,
            ((self.bounds.max_y + self.bounds.min_y) * 0.5) as f32,
            ((self.bounds.max_z + self.bounds.min_z) * 0.5) as f32,
        )
    }

    /// Calculate size dimensions
    pub fn size(&self) -> Vec3 {
        Vec3::new(
            (self.bounds.max_x - self.bounds.min_x) as f32,
            (self.bounds.max_y - self.bounds.min_y) as f32,
            (self.bounds.max_z - self.bounds.min_z) as f32,
        )
    }

    /// Get ground height for camera positioning
    pub fn ground_height(&self) -> f32 {
        self.bounds.min_y as f32
    }

    // Convenience accessors for direct bounds access
    pub fn min_x(&self) -> f64 {
        self.bounds.min_x
    }
    pub fn max_x(&self) -> f64 {
        self.bounds.max_x
    }
    pub fn min_y(&self) -> f64 {
        self.bounds.min_y
    }
    pub fn max_y(&self) -> f64 {
        self.bounds.max_y
    }
    pub fn min_z(&self) -> f64 {
        self.bounds.min_z
    }
    pub fn max_z(&self) -> f64 {
        self.bounds.max_z
    }
}

/// Check texture loading and create point cloud when ready
pub fn check_textures_loaded(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PointCloudShader>>,
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
    let colour_class_loaded = matches!(
        asset_server.get_load_state(&assets.colour_class_texture),
        Some(LoadState::Loaded)
    );
    let heightmap_loaded = matches!(
        asset_server.get_load_state(&assets.heightmap_texture),
        Some(LoadState::Loaded)
    );

    if !pos_loaded || !colour_class_loaded {
        return;
    }

    if let Some(bounds) = &assets.bounds {
        println!("Unified DDS textures loaded! Creating GPU point cloud...");

        configure_texture_sampling(&mut images, &assets);

        let mesh = create_point_index_mesh(bounds.loaded_points);
        let material = create_point_cloud_material(bounds, &assets);

        spawn_point_cloud_entity(
            &mut commands,
            &mut meshes,
            &mut materials,
            mesh,
            material,
            bounds,
        );

        // Create grid after point cloud setup
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
            println!("Grid created with unified texture support!");
        }

        assets.is_loaded = true;
    }
}

/// Configure nearest-neighbour sampling for point data textures
fn configure_texture_sampling(images: &mut ResMut<Assets<Image>>, assets: &PointCloudAssets) {
    let sampler_config = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });

    if let Some(position_image) = images.get_mut(&assets.position_texture) {
        position_image.sampler = sampler_config.clone();
    }

    if let Some(colour_class_image) = images.get_mut(&assets.colour_class_texture) {
        colour_class_image.sampler = sampler_config;
    }
}

/// Create point cloud shader material with unified texture bindings
fn create_point_cloud_material(
    bounds: &PointCloudBounds,
    assets: &PointCloudAssets,
) -> PointCloudShader {
    PointCloudShader {
        position_texture: assets.position_texture.clone(),
        colour_class_texture: assets.colour_class_texture.clone(),
        params: [
            Vec4::new(
                bounds.min_x() as f32,
                bounds.min_y() as f32,
                bounds.min_z() as f32,
                bounds.texture_size as f32,
            ),
            Vec4::new(
                bounds.max_x() as f32,
                bounds.max_y() as f32,
                bounds.max_z() as f32,
                0.0,
            ),
        ],
        polygon_data: PolygonClassificationUniform::default(),
    }
}

/// Spawn point cloud entity with unified texture rendering
fn spawn_point_cloud_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<PointCloudShader>>,
    mesh: Mesh,
    material: PointCloudShader,
    bounds: &PointCloudBounds,
) {
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        GlobalTransform::default(),
        PointCloud,
        NoFrustumCulling,
    ));

    println!(
        "Point cloud entity spawned with {} vertices",
        bounds.loaded_points
    );
    println!("Texture utilisation: {:.1}%", bounds.utilisation_percent);
    println!(
        "Colour points: {}/{}",
        bounds.colour_points, bounds.loaded_points
    );
    println!(
        "Unified texture size: {}x{}",
        bounds.texture_size, bounds.texture_size
    );
}

/// Create point index mesh for GPU texture sampling
pub fn create_point_index_mesh(point_count: usize) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::PointList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let indices: Vec<[f32; 3]> = (0..point_count).map(|i| [i as f32, 0.0, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indices);
    mesh
}

/// Sample heightmap at normalised coordinates
pub fn sample_heightmap(
    heightmap_image: &Image,
    norm_x: f32,
    norm_z: f32,
    bounds: &PointCloudBounds,
) -> f32 {
    let pixel_x = ((norm_x * (TEXTURE_SIZE - 1) as f32) as usize).min(TEXTURE_SIZE - 1);
    let pixel_y = ((norm_z * (TEXTURE_SIZE - 1) as f32) as usize).min(TEXTURE_SIZE - 1);

    let data = heightmap_image
        .data
        .as_ref()
        .expect("Heightmap image data not available");

    // Sample R32_Float texture data
    let pixel_index = (pixel_y * TEXTURE_SIZE + pixel_x) * 4; // 4 bytes per f32
    let height_bytes = &data[pixel_index..pixel_index + 4];
    let normalised_height = f32::from_le_bytes([
        height_bytes[0],
        height_bytes[1],
        height_bytes[2],
        height_bytes[3],
    ]);

    // Denormalise height to world coordinates
    bounds.min_y() as f32 + normalised_height * (bounds.max_y() - bounds.min_y()) as f32
}
