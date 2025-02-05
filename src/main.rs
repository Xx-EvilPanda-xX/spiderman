use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;

mod game;
mod input;
mod physics;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::TRACE,
            filter: "wgpu=warn,naga=warn,wgpu_hal=warn,bevy_app=warn,offset_allocator=error,bevy_render=info".to_string(),
            ..default()
        }))
        .add_systems(Startup, game::setup)
        .add_systems(Update, (game::update, input::mouse_input, input::keyboard_input))
        .add_systems(Update, physics::collion::construct_collision_trees)
        .run();
}
