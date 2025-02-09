use bevy::pbr::wireframe::WireframeConfig;
use bevy::prelude::*;
use bevy::color::palettes::css::{PURPLE, RED};
use bevy::window::PrimaryWindow;
use bevy::pbr::PointLightShadowMap;

use crate::{physics, math};
use crate::physics::collision::{ShouldRenderCollider, collision, RecursiveAABB, TriangleData};

#[derive(Component)]
pub struct Island1;

#[derive(Component, Default)]
pub struct CameraState {
    pub yaw: f32,
    pub pitch: f32,
    pub pos: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
}

#[derive(Component)]
pub struct Light1;

#[derive(Component)]
pub struct Light2;

const ROT_SPEED: f32 = 0.2;

pub fn update(
    mut islands: Query<&mut Transform, With<Island1>>,
    collision_data: Query<(&RecursiveAABB, &TriangleData, &GlobalTransform)>,
    cam: Single<&CameraState>,
    time: Res<Time>,
) {
    for mut island in &mut islands {
        *island = island.with_rotation(Quat::from_rotation_y(time.elapsed_secs() * ROT_SPEED));
    }

    let ray = math::Ray3d::new(cam.pos, cam.forward);

    for (recursive_aabb, triangle_data, transform) in &collision_data {
        let collisions = collision(ray, recursive_aabb, triangle_data, &transform.compute_matrix());
        if collisions.len() > 0 {
            debug!("{:?}", collisions);
        }
    }
}

#[allow(unused)]
pub fn debug_ecs(entities: Query<(Entity, &ShouldRenderCollider)>, mut commands: Commands) {
    debug!("------------------------------------------------------\n\n");
    for (i, name) in &entities {
        debug!("{i:#?}, {:?}", name);
        commands.entity(i).log_components();
    }
    debug!("------------------------------------------------------\n\n");
}

pub fn setup(
    mut commands: Commands,
    mut clear_color: ResMut<ClearColor>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    window.title = "Spiderman".to_string();
    window.cursor_options.visible = false;
    clear_color.0 = Color::srgb(115.0/255.0, 121.0/255.0, 121.0/255.0);
    commands.insert_resource(PointLightShadowMap { size: 2048 });
    commands.insert_resource(WireframeConfig {
        global: false,
        default_color: RED.into(),
    });

    let island_handle = server.load(GltfAssetLabel::Scene(0).from_asset("island1/Island1Export.gltf"));

    commands.spawn((
        SceneRoot(island_handle.clone()),
        Transform::from_scale(Vec3::new(0.1, 0.1, 0.1)),
        Island1,
        physics::collision::Collidable(vec![String::from("Cube.002")]),
    ));

    // let cube_handle = server.load(GltfAssetLabel::Scene(0).from_asset("cube/untitled.gltf"));

    // commands.spawn((
    //     SceneRoot(cube_handle.clone()),
    //     Transform::from_scale(Vec3::new(5.0, 5.0, 5.0)),
    //     physics::collision::Collidable(vec![String::from("Cube")]),
    // ));

    let bruh = Cuboid::new(50.0, 1.0, 50.0);

    commands.spawn((
        Mesh3d(meshes.add(bruh)),
        MeshMaterial3d(materials.add(Color::from(PURPLE))),
        Transform::from_translation(Vec3::new(0.0, -5.0, 0.0)),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 5.0, 2.0),
        Light1
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 5.0, 2.0),
        Light2
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 1.5, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraState::default()
    ));
}