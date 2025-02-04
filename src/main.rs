use bevy::input::mouse::MouseMotion;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::color::palettes::css::PURPLE;
use bevy::window::PrimaryWindow;
use bevy::window::WindowCloseRequested;
use bevy::pbr::PointLightShadowMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::TRACE,
            filter: "wgpu=warn,naga=warn,wgpu_hal=warn,bevy_app=warn,offset_allocator=error,bevy_render=info".to_string(),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (update, mouse_input, keyboard_input))
        .run();
}

#[derive(Component)]
struct Island1;

#[derive(Component, Default)]
struct CameraState {
    yaw: f32,
    pitch: f32,
    pos: Vec3,
    forward: Vec3,
    right: Vec3,
}

#[derive(Component)]
struct Light1;

#[derive(Component)]
struct Light2;

#[derive(Component)]
struct Light3;

#[derive(Component)]
struct Light4;

fn setup(
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

    let island_handle = server.load(GltfAssetLabel::Scene(0).from_asset("Island1Export.gltf"));

    commands.spawn((
        SceneRoot(island_handle),
        Transform::from_scale(Vec3::new(0.1, 0.1, 0.1)),
        Island1,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(50.0, 1.0, 50.0))),
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
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 5.0, 2.0),
        Light3
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 5.0, 2.0),
        Light4
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 1.5, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraState::default()
    ));
}

const ROT_SPEED: f32 = 0.2;
const SENSITVITY: f32 = 5.0;
const SPEED: f32 = 5.0;

fn update(
    mut islands: Query<&mut Transform, With<Island1>>,
    time: Res<Time>,
) {
    for mut island in &mut islands {
        *island = island.with_rotation(Quat::from_rotation_y(time.elapsed_secs() * ROT_SPEED));
    }
}

fn mouse_input(
    mut delta_mouse: EventReader<MouseMotion>,
    mut camera_state: Single<&mut CameraState>,
    mut camera_transform: Single<&mut Transform, With<CameraState>>,
    window: Option<Single<&mut Window, With<PrimaryWindow>>>,
    time: Res<Time>,
) {
    for d in delta_mouse.read() {
        camera_state.yaw += d.delta.x * SENSITVITY * time.delta().as_secs_f32();
        camera_state.pitch -= d.delta.y * SENSITVITY * time.delta().as_secs_f32();

        camera_state.pitch = camera_state.pitch.clamp(-89.9, 89.9);
    }

    if let Some(mut window) = window {
        let x = window.resolution.width() as f64 * 0.5;
        let y = window.resolution.height() as f64 * 0.5;
        window.set_physical_cursor_position(Some(DVec2::new(x, y)));
    }

    eval_camera_vecs(&mut camera_state);
    camera_transform.look_to(camera_state.forward, Vec3::Y);
}

fn eval_camera_vecs(camera_state: &mut CameraState) {
    let forward = Vec3::new(
        camera_state.yaw.to_radians().cos() * camera_state.pitch.to_radians().cos(),
        camera_state.pitch.to_radians().sin(),
        camera_state.yaw.to_radians().sin() * camera_state.pitch.to_radians().cos(),
    ).normalize();

    let right = forward.cross(Vec3::Y);

    camera_state.forward = forward;
    camera_state.right = right;
}

fn keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Option<Single<Entity, With<PrimaryWindow>>>,
    mut writer: EventWriter<WindowCloseRequested>,
    mut camera_state: Single<&mut CameraState>,
    mut set: ParamSet<(
        Single<&mut Transform, With<CameraState>>,
        Single<&mut Transform, With<Light1>>,
        Single<&mut Transform, With<Light2>>,
        Single<&mut Transform, With<Light3>>,
        Single<&mut Transform, With<Light4>>,
    )>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if let Some(window) = window {
            writer.send(WindowCloseRequested { window: *window });
        }
    }

    let speed = SPEED * time.delta().as_secs_f32();

    if input.pressed(KeyCode::KeyW) {
        camera_state.pos.x += camera_state.forward.x * speed;
        camera_state.pos.y += camera_state.forward.y * speed;
        camera_state.pos.z += camera_state.forward.z * speed;
    }

    if input.pressed(KeyCode::KeyS) {
        camera_state.pos.x -= camera_state.forward.x * speed;
        camera_state.pos.y -= camera_state.forward.y * speed;
        camera_state.pos.z -= camera_state.forward.z * speed;
    }

    if input.pressed(KeyCode::KeyD) {
        camera_state.pos.x += camera_state.right.x * speed;
        camera_state.pos.y += camera_state.right.y * speed;
        camera_state.pos.z += camera_state.right.z * speed;
    }

    if input.pressed(KeyCode::KeyA) {
        camera_state.pos.x -= camera_state.right.x * speed;
        camera_state.pos.y -= camera_state.right.y * speed;
        camera_state.pos.z -= camera_state.right.z * speed;
    }

    if input.pressed(KeyCode::Space) {
        camera_state.pos.y += speed;
    }

    if input.pressed(KeyCode::ShiftLeft) {
        camera_state.pos.y -= speed;
    }

    if input.pressed(KeyCode::Digit1) {
        set.p1().translation = set.p0().translation.clone();
    }

    if input.pressed(KeyCode::Digit2) {
        set.p2().translation = set.p0().translation.clone();
    }

    if input.pressed(KeyCode::Digit3) {
        set.p3().translation = set.p0().translation.clone();
    }

    if input.pressed(KeyCode::Digit4) {
        set.p4().translation = set.p0().translation.clone();
    }

    set.p0().translation = camera_state.pos;
}
