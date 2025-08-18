use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use super::{
    point_cloud::sample_heightmap,
    point_cloud::{PointCloudAssets, PointCloudBounds},
};

#[derive(Resource)]
pub struct ViewportCamera {
    pub focus_point: Vec3,
    pub height: f32,
    pub rotation: Quat,
    pub is_panning: bool,
    pub last_mouse_pos: Vec2,
    pub ground_height: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub pan_start_world_point: Option<Vec3>,
}

impl ViewportCamera {
    pub fn new(center: Vec3, ground_height: f32) -> Self {
        let size = Vec3::new(100.0, 50.0, 100.0);
        Self {
            focus_point: center,
            height: size.length() * 0.8,
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
            ground_height,
            rotation: Quat::default(),
            pitch: -0.6,
            yaw: 0.0,
            pan_start_world_point: None,
        }
    }

    pub fn with_bounds(bounds: &PointCloudBounds) -> Self {
        let center = bounds.center();
        let size = bounds.size();
        let ground_height = bounds.ground_height();

        Self {
            focus_point: center,
            height: size.length() * 0.2,
            rotation: Quat::from_rotation_x(-0.6),
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
            ground_height,
            pitch: -0.6,
            yaw: 0.0,
            pan_start_world_point: None,
        }
    }

    pub fn update_rotation(&mut self) {
        self.pitch = self.pitch.clamp(-1.5, -0.1);
        self.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    pub fn update_transform(&self) -> Transform {
        let offset = self.rotation * Vec3::new(0.0, 0.0, self.height);
        let position = self.focus_point + offset;
        Transform::from_translation(position).looking_at(self.focus_point, Vec3::Y)
    }

    pub fn mouse_to_ground_plane(
        &self,
        cursor_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        heightmap_image: Option<&Image>,
        bounds: Option<&PointCloudBounds>,
    ) -> Option<Vec3> {
        let ray = camera
            .viewport_to_world(camera_transform, cursor_pos)
            .ok()?;

        // If heightmap is available, use it
        if let (Some(heightmap), Some(bounds)) = (heightmap_image, bounds) {
            let mut t = 0.0;
            let step_size = 1.0;
            let max_distance = 1000.0;

            while t < max_distance {
                let test_point = ray.origin + ray.direction * t;

                let norm_x = (test_point.x - bounds.bounds.min_x as f32)
                    / (bounds.bounds.max_x - bounds.bounds.min_x) as f32;
                let norm_z = (test_point.z - bounds.bounds.min_z as f32)
                    / (bounds.bounds.max_z - bounds.bounds.min_z) as f32;

                if norm_x >= 0.0 && norm_x <= 1.0 && norm_z >= 0.0 && norm_z <= 1.0 {
                    let terrain_height = sample_heightmap(heightmap, norm_x, norm_z, bounds);

                    if test_point.y <= terrain_height {
                        return Some(Vec3::new(test_point.x, terrain_height, test_point.z));
                    }
                }
                t += step_size;
            }
        }

        // Fallback to flat ground plane
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

    pub fn pan_to_keep_world_point_under_mouse(
        &mut self,
        mouse_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) {
        if let Some(target_world_point) = self.pan_start_world_point {
            // Changed from Option to Result pattern matching
            if let Ok(current_screen_pos) =
                camera.world_to_viewport(camera_transform, target_world_point)
            {
                let screen_delta = mouse_pos - current_screen_pos;
                let camera_right = camera_transform.right();
                let camera_up = camera_transform.up();
                let distance_to_ground =
                    (camera_transform.translation().y - self.ground_height).abs();
                let scale = distance_to_ground * 0.001;
                let world_movement =
                    camera_right * -screen_delta.x * scale + camera_up * screen_delta.y * scale;
                self.focus_point += world_movement;
            }
        }
    }
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self {
            focus_point: Vec3::ZERO,
            height: 100.0,
            rotation: Quat::default(),
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
            ground_height: 0.0,
            pitch: -0.6,
            yaw: 0.0,
            pan_start_world_point: None,
        }
    }
}

pub fn camera_controller(
    mut camera_query: Query<(&mut Transform, &GlobalTransform, &Camera), With<Camera3d>>,
    mut maps_camera: ResMut<ViewportCamera>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor_moved: EventReader<CursorMoved>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
) {
    if let Ok((mut camera_transform, global_transform, camera)) = camera_query.single_mut() {
        // Update cursor position
        for cursor in cursor_moved.read() {
            maps_camera.last_mouse_pos = cursor.position;
        }

        // Handle zoom (scroll wheel)
        for scroll in scroll_events.read() {
            let zoom_factor = if scroll.y > 0.0 { 0.9 } else { 1.1 };
            maps_camera.height *= zoom_factor;
            maps_camera.height = maps_camera.height.clamp(5.0, 5000.0);
        }

        // Collect mouse motion
        let total_motion: Vec2 = mouse_motion.read().map(|motion| motion.delta).sum();

        let is_panning = mouse_button.pressed(MouseButton::Middle);
        let is_rotating = mouse_button.pressed(MouseButton::Right);

        // Handle panning - fixed directions
        if is_panning && total_motion != Vec2::ZERO {
            let sensitivity = maps_camera.height * 0.001;
            let yaw_rot = Quat::from_rotation_y(maps_camera.yaw);
            let right = yaw_rot * Vec3::X;
            let forward = yaw_rot * Vec3::Z;

            maps_camera.focus_point += right * -total_motion.x * sensitivity;
            maps_camera.focus_point += forward * -total_motion.y * sensitivity;
        }

        // Handle keyboard rotation (works always)
        let mut rotation_input = 0.0;
        if keyboard.pressed(KeyCode::KeyA) {
            rotation_input -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            rotation_input += 1.0;
        }

        // Apply rotation if keys are pressed
        if rotation_input != 0.0 {
            let rotation_speed = 1.0 * time.delta_secs();
            maps_camera.yaw += rotation_input * rotation_speed;
        }

        // Follow mouse with right click
        if is_rotating {
            if let Some(pivot_point) = maps_camera.mouse_to_ground_plane(
                maps_camera.last_mouse_pos,
                camera,
                global_transform,
                images.get(&assets.heightmap_texture),
                assets.bounds.as_ref(),
            ) {
                let movement_speed = 2.0 * time.delta_secs();
                maps_camera.focus_point = maps_camera.focus_point.lerp(pivot_point, movement_speed);
            }
        }

        // Simple camera positioning
        let yaw_rot = Quat::from_rotation_y(maps_camera.yaw);
        let horizontal_offset = yaw_rot * Vec3::new(0.0, 0.0, maps_camera.height * 0.5);
        let target_pos = maps_camera.focus_point
            + Vec3::new(horizontal_offset.x, maps_camera.height, horizontal_offset.z);

        let target_transform =
            Transform::from_translation(target_pos).looking_at(maps_camera.focus_point, Vec3::Y);

        // Smooth interpolation
        let lerp_speed = 12.0 * time.delta_secs();
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_transform.translation, lerp_speed.min(1.0));
        camera_transform.rotation = camera_transform
            .rotation
            .slerp(target_transform.rotation, lerp_speed.min(1.0));
    }
}
