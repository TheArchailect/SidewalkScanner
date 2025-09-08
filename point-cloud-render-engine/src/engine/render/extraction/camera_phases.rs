use bevy::prelude::*;

use crate::engine::point_cloud_render_pipeline::PointCloudPhase;

pub fn extract_camera_phases(
    mut point_cloud_phases: ResMut<
        bevy::render::render_phase::ViewSortedRenderPhases<PointCloudPhase>,
    >,
    cameras: bevy::render::Extract<Query<(Entity, &Camera), With<Camera3d>>>,
    mut live_entities: Local<std::collections::HashSet<bevy::render::view::RetainedViewEntity>>,
) {
    live_entities.clear();
    for (main_entity, camera) in &cameras {
        if !camera.is_active {
            continue;
        }

        let retained_view_entity =
            bevy::render::view::RetainedViewEntity::new(main_entity.into(), None, 0);
        point_cloud_phases.insert_or_clear(retained_view_entity);
        live_entities.insert(retained_view_entity);
    }
    point_cloud_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
}
