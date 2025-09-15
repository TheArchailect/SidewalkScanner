use crate::engine::assets::bounds::PointCloudBounds;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::scene::heightmap::sample_heightmap_bilinear;
use crate::tools::asset_manager::SelectionLock;
use bevy::input::mouse::MouseScrollUnit;
use bevy::math::EulerRot;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

// TODO: Fix cursor "disappearing"
// - Add keybind functionality to toggle modes and rebind keys

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    Fly,
    Orbit,
}

// Temporary orbit spawn constants
const ORBIT_SPAWN: Vec3 = Vec3::new(-6.422367, 50.903121, 10.592193);
const ORBIT_YAW: f32 = 0.0;
const ORBIT_PITCH: f32 = -0.6;
const ORBIT_DISTANCE: f32 = 20.0;

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
    // Add smoothing for intersection
    pub last_intersection: Option<Vec3>,
    pub intersection_smooth_factor: f32,

    pub mode: CameraMode,
    pub fly_base_speed: f32,
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
            last_intersection: None,
            intersection_smooth_factor: 0.1, // Adjust for smoothness vs responsiveness
            mode: CameraMode::Fly,
            fly_base_speed: 20.0,
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
            last_intersection: None,
            intersection_smooth_factor: 0.15,
            mode: CameraMode::Fly,
            fly_base_speed: 20.0,
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
            last_intersection: None,
            intersection_smooth_factor: 0.15,
            mode: CameraMode::Fly,
            fly_base_speed: 20.0,
        }
    }
}

pub fn camera_controller(
    mut camera_query: Query<(&mut Transform, &GlobalTransform, &Camera), With<Camera3d>>,
    mut maps_camera: ResMut<ViewportCamera>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut cursor_moved: EventReader<CursorMoved>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    assets: Res<PointCloudAssets>,
    manifests: Res<Assets<SceneManifest>>,
    lock: Option<Res<SelectionLock>>,
) {
    // lock for rotating bounds
    if let Some(lock) = lock {
        if lock.active {
            return; // skip zoom while a bound is selected
        }
    }

    if let Ok((mut camera_transform, global_transform, camera)) = camera_query.single_mut() {
        // Update cursor position
        for cursor in cursor_moved.read() {
            maps_camera.last_mouse_pos = cursor.position;
        }

        // Toggle mode with "F"
        if keyboard.just_pressed(KeyCode::KeyF) {
            let going_to_orbit = matches!(maps_camera.mode, CameraMode::Fly);
            maps_camera.mode = if going_to_orbit {
                CameraMode::Orbit
            } else {
                CameraMode::Fly
            };

            if going_to_orbit {
                // reset orbit state
                maps_camera.focus_point = ORBIT_SPAWN;
                maps_camera.yaw = ORBIT_YAW;
                maps_camera.pitch = ORBIT_PITCH;
                maps_camera.height = ORBIT_DISTANCE;

                let orbit_rot =
                    Quat::from_euler(EulerRot::YXZ, maps_camera.yaw, maps_camera.pitch, 0.0);
                let horiz = orbit_rot * Vec3::new(0.0, 0.0, maps_camera.height * 0.5);
                let target_pos =
                    maps_camera.focus_point + Vec3::new(horiz.x, maps_camera.height, horiz.z);

                camera_transform.translation = target_pos;
                camera_transform.rotation = Transform::from_translation(target_pos)
                    .looking_at(maps_camera.focus_point, Vec3::Y)
                    .rotation;
            }
        }

        // Read mouse motion
        let mouse_delta: Vec2 = mouse_motion.read().map(|m| m.delta).sum();

        // Obtain bounds
        let _maybe_bounds = assets.get_bounds(&manifests);

        // Accumulate scroll (pixel/line)
        let mut scroll_accum = 0.0;
        for ev in scroll_events.read() {
            scroll_accum += match ev.unit {
                MouseScrollUnit::Line => ev.y * 1.0,
                MouseScrollUnit::Pixel => ev.y * 0.05,
            };
        }

        match maps_camera.mode {
            CameraMode::Fly => {
                // Look with right mouse button
                if mouse_button.pressed(MouseButton::Right) && mouse_delta != Vec2::ZERO {
                    let yaw_sens = 0.003;
                    let pitch_sens = 0.003;
                    maps_camera.yaw += -mouse_delta.x * yaw_sens;
                    maps_camera.pitch += -mouse_delta.y * pitch_sens;
                    maps_camera.pitch = maps_camera.pitch.clamp(-1.55, 1.55);
                }

                // Zoom
                if scroll_accum.abs() > f32::EPSILON {
                    let dolly_speed = (maps_camera.height * 0.2).clamp(0.5, 500.0);
                    let view_rot =
                        Quat::from_euler(EulerRot::YXZ, maps_camera.yaw, maps_camera.pitch, 0.0);
                    let forward = (view_rot * Vec3::Z).normalize();
                    maps_camera.focus_point -= forward * (scroll_accum * dolly_speed);
                }

                // Movement WASD + Q/W (up/down)
                let mut move_input = Vec3::ZERO;
                if keyboard.pressed(KeyCode::KeyW) {
                    move_input.z -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyS) {
                    move_input.z += 1.0;
                }
                if keyboard.pressed(KeyCode::KeyD) {
                    move_input.x += 1.0;
                }
                if keyboard.pressed(KeyCode::KeyA) {
                    move_input.x -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyE) {
                    move_input.y += 1.0;
                } // Up
                if keyboard.pressed(KeyCode::KeyQ) {
                    move_input.y -= 1.0;
                } // Down

                if move_input != Vec3::ZERO {
                    let view_rot =
                        Quat::from_euler(EulerRot::YXZ, maps_camera.yaw, maps_camera.pitch, 0.0);
                    let forward = (view_rot * Vec3::Z).normalize();
                    let right = (view_rot * Vec3::X).normalize();
                    let up = Vec3::Y;

                    let mut speed = maps_camera.fly_base_speed;
                    if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                        speed *= 3.5;
                    }
                    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                        speed *= 0.25;
                    }

                    let world_delta =
                        right * move_input.x + up * move_input.y + forward * move_input.z;
                    maps_camera.focus_point += world_delta.normalize() * speed * time.delta_secs();
                }

                // Camera transformation
                let target_rot =
                    Quat::from_euler(EulerRot::YXZ, maps_camera.yaw, maps_camera.pitch, 0.0);
                let target_pos = maps_camera.focus_point;
                let s = (12.0 * time.delta_secs()).min(1.0);
                camera_transform.translation = camera_transform.translation.lerp(target_pos, s);
                camera_transform.rotation = camera_transform.rotation.slerp(target_rot, s);
            }

            CameraMode::Orbit => {
                // AD rotate
                let mut yaw_input = 0.0;
                if keyboard.pressed(KeyCode::KeyA) {
                    yaw_input -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyD) {
                    yaw_input += 1.0;
                }
                let yaw_speed = 1.4;
                maps_camera.yaw += yaw_input * yaw_speed * time.delta_secs();

                // Right mouse pan
                if mouse_button.pressed(MouseButton::Right) && mouse_delta != Vec2::ZERO {
                    let sensitivity = maps_camera.height * 0.0025;
                    let yaw_only = Quat::from_rotation_y(maps_camera.yaw);
                    let right = yaw_only * Vec3::X;
                    let forward = yaw_only * Vec3::Z;
                    maps_camera.focus_point += right * -mouse_delta.x * sensitivity;
                    maps_camera.focus_point += forward * -mouse_delta.y * sensitivity;
                }

                // Scroll zoom
                if scroll_accum.abs() > f32::EPSILON {
                    // Decide new height
                    let zoom_factor = if scroll_accum > 0.0 { 0.90 } else { 1.10 };
                    let old_h = maps_camera.height;
                    let new_h = (old_h * zoom_factor).clamp(2.0, 5000.0);

                    // Get ground hit under the cursor
                    let mut hit_world: Option<Vec3> = None;
                    if let Some(bounds) = _maybe_bounds {
                        let cursor_pos = maps_camera.last_mouse_pos;
                        let heightmap_opt = None;

                        hit_world = maps_camera.mouse_to_ground_plane(
                            cursor_pos,
                            camera,
                            global_transform,
                            heightmap_opt,
                            &bounds,
                        );
                    }

                    // Move focus toward the cursor
                    if let Some(hit) = hit_world {
                        let k = ((old_h - new_h) / old_h).clamp(0.0, 1.0);
                        maps_camera.focus_point = maps_camera.focus_point.lerp(hit, k);
                    }
                    maps_camera.height = new_h;
                }

                // Orbit placement
                let orbit_rot =
                    Quat::from_euler(EulerRot::YXZ, maps_camera.yaw, maps_camera.pitch, 0.0);
                let horizontal_offset = orbit_rot * Vec3::new(0.0, 0.0, maps_camera.height * 0.5);
                let target_pos = maps_camera.focus_point
                    + Vec3::new(horizontal_offset.x, maps_camera.height, horizontal_offset.z);
                let target_rot = Transform::from_translation(target_pos)
                    .looking_at(maps_camera.focus_point, Vec3::Y)
                    .rotation;

                let s = (12.0 * time.delta_secs()).min(1.0);
                camera_transform.translation = camera_transform.translation.lerp(target_pos, s);
                camera_transform.rotation = camera_transform.rotation.slerp(target_rot, s);
            }
        }
    }
}
