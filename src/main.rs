use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuSettings, WgpuFeatures};

mod game;
mod input;
mod physics;
mod math;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(LogPlugin {
                level: Level::TRACE,
                filter: "wgpu=warn,naga=warn,wgpu_hal=warn,bevy_app=warn,offset_allocator=error,bevy_render=info".to_string(),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugins(bevy::pbr::wireframe::WireframePlugin)
        .add_systems(Startup, game::setup)
        .add_systems(Update, (game::update, input::mouse_input, input::keyboard_input))
        .add_systems(Update, (physics::collision::construct_collision_trees, physics::collision::add_collider_wireframes))
        // .add_systems(Update, game::debug_ecs)
        .run();
}
