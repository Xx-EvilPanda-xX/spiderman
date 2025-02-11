use bevy::input::mouse::MouseMotion;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowCloseRequested};
use super::game::{CameraState, Light1, Light2};

const SENSITVITY: f32 = 0.05;
const SPEED: f32 = 5.0;

pub fn mouse_input(
    mut delta_mouse: EventReader<MouseMotion>,
    mut camera_state: Single<&mut CameraState>,
    mut camera_transform: Single<&mut Transform, With<CameraState>>,
    window: Option<Single<&mut Window, With<PrimaryWindow>>>,
) {
    for d in delta_mouse.read() {
        camera_state.yaw += d.delta.x * SENSITVITY;
        camera_state.pitch -= d.delta.y * SENSITVITY;

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

pub fn keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Option<Single<Entity, With<PrimaryWindow>>>,
    mut writer: EventWriter<WindowCloseRequested>,
    mut camera_state: Single<&mut CameraState>,
    mut set: ParamSet<(
        Single<&mut Transform, With<CameraState>>,
        Single<&mut Transform, With<Light1>>,
        Single<&mut Transform, With<Light2>>,
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

    if input.pressed(KeyCode::KeyV) {
        debug!("{:?}", camera_state.pos);
    }

    if input.pressed(KeyCode::Digit1) {
        set.p1().translation = set.p0().translation.clone();
    }

    if input.pressed(KeyCode::Digit2) {
        set.p2().translation = set.p0().translation.clone();
    }

    set.p0().translation = camera_state.pos;
}