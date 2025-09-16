use bevy::prelude::*;
use super::state::*;
use super::ray::ray_hits_obb;

// Toggles selection of placed bounds on left mouse click
pub fn toggle_select_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    q_bounds: Query<(Entity, &GlobalTransform, &BoundsSize, Option<&Selected>), With<PlacedBounds>>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((cam_xf, camera)) = cameras.single() else { return; };

    let Ok(ray) = camera.viewport_to_world(cam_xf, cursor_pos) else { return; };
    let origin = ray.origin;
    let dir = ray.direction.as_vec3();

    let mut best: Option<(Entity, f32, bool)> = None;
    for (e, xf, BoundsSize(size), selected) in &q_bounds {
        if let Some(t) = ray_hits_obb(origin, dir, *xf, *size) {
            if t > 0.0 && (best.is_none() || t < best.unwrap().1) {
                best = Some((e, t, selected.is_some()));
            }
        }
    }

    if let Some((hit_e, _t, was_selected)) = best {
        // Deselect all
        for (e, _, _, sel) in &q_bounds {
            if sel.is_some() {
                commands.entity(e).remove::<Selected>();
                commands.entity(e).remove::<ActiveRotating>();
                commands.entity(e).insert(bevy::pbr::wireframe::WireframeColor { color: Color::WHITE });
            }
        }
        // Select hit
        if !was_selected {
            commands.entity(hit_e).insert(Selected);
            commands.entity(hit_e).insert(ActiveRotating);
            commands.entity(hit_e).insert(bevy::pbr::wireframe::WireframeColor { color: Color::srgb(1.0, 1.0, 0.0) });
        }
    }
}

pub fn reflect_selection_lock(q_selected: Query<(), With<Selected>>, mut lock: ResMut<SelectionLock>) {
    lock.active = !q_selected.is_empty();
}

// Deselect all on Escape key press
pub fn deselect_on_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    q_bounds: Query<(Entity, Option<&Selected>), With<PlacedBounds>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        for (e, sel) in &q_bounds {
            if sel.is_some() {
                commands.entity(e).remove::<Selected>();
                commands.entity(e).remove::<ActiveRotating>();
                commands.entity(e).insert(bevy::pbr::wireframe::WireframeColor { color: Color::WHITE });
            }
        }
    }
}

