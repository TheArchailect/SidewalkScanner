use crate::engine::render::pipeline::point_cloud_render_pipeline::PointCloudRenderable;
use bevy::prelude::*;

use crate::engine::core::app_state::PipelineDebugState;

pub fn debug_pipeline_state(
    debug_state: Res<PipelineDebugState>,
    point_cloud_query: Query<Entity, With<PointCloudRenderable>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        let pc_entities = point_cloud_query.iter().count();

        println!("=== CUSTOM PHASE DEBUG STATE ===");
        println!("Point cloud entities in main world: {}", pc_entities);
        println!(
            "Entities queued for rendering: {}",
            debug_state.entities_queued
        );
        println!("Mesh instances found: {}", debug_state.mesh_instances_found);
        println!(
            "Pipeline specializations: {}",
            debug_state.pipeline_specializations
        );
        println!("Phase items added: {}", debug_state.phase_items_added);
        println!("Views with phases: {}", debug_state.views_with_phases);

        // Critical assertions
        assert!(pc_entities > 0, "No point cloud entities in main world");
        assert!(
            debug_state.entities_queued > 0,
            "No entities queued for rendering"
        );
        assert!(
            debug_state.mesh_instances_found > 0,
            "No mesh instances found"
        );
        assert!(debug_state.phase_items_added > 0, "No phase items added");
        assert!(debug_state.views_with_phases > 0, "No views have phases");

        println!("All assertions passed!");
    }
}
