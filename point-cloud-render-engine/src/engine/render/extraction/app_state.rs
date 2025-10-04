use crate::engine::core::app_state::AppState;
use bevy::prelude::*;

pub fn extract_app_state(
    main_world: bevy::render::Extract<Res<State<AppState>>>,
    mut commands: Commands,
) {
    commands.insert_resource(State::new(*main_world.get()));
}
