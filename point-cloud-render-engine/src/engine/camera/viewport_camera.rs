use crate::engine::assets::bounds::PointCloudBounds;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::scene::heightmap::sample_heightmap_bilinear;
use crate::tools::asset_manager::interactions::ScrollCapture;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Resource)]
pub struct ViewportCamera {
    pub focus_point: Vec3,
    pub height: f32,
    pub last_mouse_pos: Vec2,
    pub ground_height: f32,
    pub yaw: f32,
    pub pitch: f32,
    // Add smoothing for intersection
    pub last_intersection: Option<Vec3>,
    pub intersection_smooth_factor: f32,
}

impl ViewportCamera {
    pub fn with_bounds(bounds: &PointCloudBounds) -> Self {
        let center = bounds.center();
        let size = bounds.size();
        // let ground_height = bounds.ground_height();
        let ground_height = 0.0;

        Self {
            focus_point: center,
            height: size.length() * 0.2,
            last_mouse_pos: Vec2::ZERO,
            ground_height,
            yaw: 0.0,
            pitch: -0.785,
            last_intersection: None,
            intersection_smooth_factor: 0.3,
        }
    }

    pub fn mouse_to_ground_plane(
        &mut self,
        cursor_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        heightmap_image: Option<&Image>,
        bounds: &PointCloudBounds,
    ) -> Option<Vec3> {
        let ray = camera
            .viewport_to_world(camera_transform, cursor_pos)
            .ok()?;

        let intersection = if let Some(heightmap) = heightmap_image {
            self.precise_heightmap_intersection(&ray, heightmap, bounds)
        } else {
            self.flat_plane_intersection(&ray)
        };

        // let intersection = self.flat_plane_intersection(&ray);

        // Apply temporal smoothing to reduce jitter
        match (intersection, self.last_intersection) {
            (Some(new_pos), Some(last_pos)) => {
                let smoothed = last_pos.lerp(new_pos, self.intersection_smooth_factor);
                self.last_intersection = Some(smoothed);
                Some(smoothed)
            }
            (Some(new_pos), None) => {
                self.last_intersection = Some(new_pos);
                Some(new_pos)
            }
            _ => None,
        }
    }

    fn precise_heightmap_intersection(
        &self,
        ray: &Ray3d,
        heightmap_image: &Image,
        bounds: &PointCloudBounds,
    ) -> Option<Vec3> {
        // Adaptive step size based on camera height for better precision
        let base_step = (self.height * 0.01).clamp(0.1, 2.0);
        let mut t = 0.0;
        let max_distance = self.height * 3.0;
        let mut last_height_diff = f32::INFINITY;

        while t < max_distance {
            let test_point = ray.origin + ray.direction * t;

            // Check if point is within bounds
            let norm_x = (test_point.x - bounds.bounds.min_x as f32)
                / (bounds.bounds.max_x - bounds.bounds.min_x) as f32;
            let norm_z = (test_point.z - bounds.bounds.min_z as f32)
                / (bounds.bounds.max_z - bounds.bounds.min_z) as f32;

            if norm_x >= 0.0 && norm_x <= 1.0 && norm_z >= 0.0 && norm_z <= 1.0 {
                let terrain_height =
                    sample_heightmap_bilinear(heightmap_image, norm_x, norm_z, bounds);
                let height_diff = test_point.y - terrain_height;

                // Check for intersection (ray crosses terrain)
                if height_diff <= 0.0 {
                    // Refine intersection with binary search for sub-pixel accuracy
                    if last_height_diff != f32::INFINITY && last_height_diff > 0.0 {
                        let refined_t = self.binary_search_intersection(
                            ray,
                            t - base_step,
                            t,
                            heightmap_image,
                            bounds,
                            5, // iterations
                        );
                        let final_point = ray.origin + ray.direction * refined_t;
                        let final_norm_x = (final_point.x - bounds.bounds.min_x as f32)
                            / (bounds.bounds.max_x - bounds.bounds.min_x) as f32;
                        let final_norm_z = (final_point.z - bounds.bounds.min_z as f32)
                            / (bounds.bounds.max_z - bounds.bounds.min_z) as f32;
                        let final_height = sample_heightmap_bilinear(
                            heightmap_image,
                            final_norm_x,
                            final_norm_z,
                            bounds,
                        );

                        return Some(Vec3::new(final_point.x, final_height, final_point.z));
                    } else {
                        return Some(Vec3::new(test_point.x, terrain_height, test_point.z));
                    }
                }
                last_height_diff = height_diff;
            }

            // Adaptive step size - smaller steps when close to intersection
            let step_size =
                if last_height_diff != f32::INFINITY && last_height_diff < base_step * 2.0 {
                    base_step * 0.1 // Fine steps near intersection
                } else {
                    base_step
                };

            t += step_size;
        }

        None
    }

    fn binary_search_intersection(
        &self,
        ray: &Ray3d,
        t_start: f32,
        t_end: f32,
        heightmap_image: &Image,
        bounds: &PointCloudBounds,
        iterations: usize,
    ) -> f32 {
        let mut low = t_start;
        let mut high = t_end;

        for _ in 0..iterations {
            let mid = (low + high) * 0.5;
            let test_point = ray.origin + ray.direction * mid;

            let norm_x = (test_point.x - bounds.bounds.min_x as f32)
                / (bounds.bounds.max_x - bounds.bounds.min_x) as f32;
            let norm_z = (test_point.z - bounds.bounds.min_z as f32)
                / (bounds.bounds.max_z - bounds.bounds.min_z) as f32;

            if norm_x >= 0.0 && norm_x <= 1.0 && norm_z >= 0.0 && norm_z <= 1.0 {
                let terrain_height =
                    sample_heightmap_bilinear(heightmap_image, norm_x, norm_z, bounds);

                if test_point.y > terrain_height {
                    low = mid;
                } else {
                    high = mid;
                }
            }
        }

        (low + high) * 0.5
    }

    fn flat_plane_intersection(&self, ray: &Ray3d) -> Option<Vec3> {
        let plane_y = self.ground_height;
        if ray.direction.y.abs() < 0.001 {
            return None;
        }
        let t = (plane_y - ray.origin.y) / ray.direction.y;
        if t > 0.0 {
            Some(ray.origin + ray.direction * t)
        } else {
            None
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn camera_controller(
    mut camera_query: Query<(&mut Transform, &GlobalTransform, &Camera), With<Camera3d>>,
    mut viewport_camera: ResMut<ViewportCamera>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut cursor_moved: EventReader<CursorMoved>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
    cap: Res<ScrollCapture>,
) {
    if let Ok((mut camera_transform, global_transform, camera)) = camera_query.single_mut() {
        // Update cursor position if we're not rotating some asset
        if !cap.lock_zoom_this_frame {
            for cursor in cursor_moved.read() {
                viewport_camera.last_mouse_pos = cursor.position;
            }

            // Handle zoom (scroll wheel)
            for scroll in scroll_events.read() {
                let zoom_factor = if scroll.y > 0.0 { 0.9 } else { 1.1 };
                viewport_camera.height = lerp(
                    viewport_camera.height,
                    viewport_camera.height * zoom_factor,
                    viewport_camera.intersection_smooth_factor,
                );
            }
        }

        let total_motion: Vec2 = mouse_motion.read().map(|motion| motion.delta).sum();
        let is_panning = mouse_button.pressed(MouseButton::Middle);
        let is_following_mouse =
            keyboard.pressed(KeyCode::Space) | mouse_button.pressed(MouseButton::Right);

        // Handle panning - fixed directions
        if is_panning && total_motion != Vec2::ZERO {
            let sensitivity = viewport_camera.height * 0.001;
            let yaw_rot = Quat::from_rotation_y(viewport_camera.yaw);
            let right = yaw_rot * Vec3::X;
            let forward = yaw_rot * Vec3::Z;

            viewport_camera.focus_point += right * -total_motion.x * sensitivity;
            viewport_camera.focus_point += forward * -total_motion.y * sensitivity;
        }

        // Handle keyboard rotation (works always)
        let mut rotation_input = 0.0;
        if keyboard.pressed(KeyCode::KeyE) {
            rotation_input -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyQ) {
            rotation_input += 1.0;
        }

        // Apply rotation if keys are pressed
        if rotation_input != 0.0 {
            let rotation_speed = 1.0 * time.delta_secs();
            viewport_camera.yaw += rotation_input * rotation_speed;
        }

        let Some(bounds) = assets.get_bounds(&manifests) else {
            return;
        };

        // Control yaw
        if keyboard.pressed(KeyCode::PageDown) {
            let rotation_speed = 1.0 * time.delta_secs();
            viewport_camera.pitch = (viewport_camera.pitch + rotation_speed).clamp(-1.4, -0.1);
        }
        if keyboard.pressed(KeyCode::PageUp) {
            let rotation_speed = 1.0 * time.delta_secs();
            viewport_camera.pitch = (viewport_camera.pitch - rotation_speed).clamp(-1.4, -0.1);
        }

        // WASD camera focus point update
        let was_speed_mult = viewport_camera.height * 0.35;
        let movement_speed = 2.0 * time.delta_secs();
        if keyboard.pressed(KeyCode::KeyW) {
            let yaw_rot = Quat::from_rotation_y(viewport_camera.yaw);
            let forward = yaw_rot * Vec3::Z;
            let new_position = viewport_camera.focus_point + forward * -1.0 * was_speed_mult;
            viewport_camera.focus_point = viewport_camera
                .focus_point
                .lerp(new_position, movement_speed);
        }
        if keyboard.pressed(KeyCode::KeyA) {
            let yaw_rot = Quat::from_rotation_y(viewport_camera.yaw);
            let right = yaw_rot * Vec3::X;
            let new_position = viewport_camera.focus_point + right * -1.0 * was_speed_mult;
            viewport_camera.focus_point = viewport_camera
                .focus_point
                .lerp(new_position, movement_speed);
        }
        if keyboard.pressed(KeyCode::KeyS) {
            let yaw_rot = Quat::from_rotation_y(viewport_camera.yaw);
            let forward = yaw_rot * Vec3::Z;
            let new_position = viewport_camera.focus_point + forward * 1.0 * was_speed_mult;
            viewport_camera.focus_point = viewport_camera
                .focus_point
                .lerp(new_position, movement_speed);
        }
        if keyboard.pressed(KeyCode::KeyD) {
            let yaw_rot = Quat::from_rotation_y(viewport_camera.yaw);
            let right = yaw_rot * Vec3::X;
            let new_position = viewport_camera.focus_point + right * 1.0 * was_speed_mult;
            viewport_camera.focus_point = viewport_camera
                .focus_point
                .lerp(new_position, movement_speed);
        }

        // Follow mouse with spacebar
        if is_following_mouse {
            let last_pos = viewport_camera.last_mouse_pos;
            if let Some(pivot_point) = viewport_camera.mouse_to_ground_plane(
                last_pos,
                camera,
                global_transform,
                images.get(&assets.heightmap_texture),
                &bounds,
            ) {
                viewport_camera.focus_point = viewport_camera
                    .focus_point
                    .lerp(pivot_point, movement_speed);
            }
        }

        // Simple camera positioning
        let yaw_rot = Quat::from_rotation_y(viewport_camera.yaw);
        let pitch_rot = Quat::from_rotation_x(viewport_camera.pitch);
        // Combine rotations: yaw around Y axis, then pitch around local X axis
        let combined_rot = yaw_rot * pitch_rot;

        let offset = combined_rot * Vec3::new(0.0, 0.0, viewport_camera.height);
        let target_pos = viewport_camera.focus_point + offset;

        let target_transform = Transform::from_translation(target_pos)
            .looking_at(viewport_camera.focus_point, Vec3::Y);

        // Smooth interpolation
        let lerp_speed = 24.0 * time.delta_secs();
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_transform.translation, lerp_speed.min(1.0));
        camera_transform.rotation = camera_transform
            .rotation
            .slerp(target_transform.rotation, lerp_speed.min(1.0));
    }
}
