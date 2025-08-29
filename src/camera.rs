use bevy::{
    math::*,
    window::*,
    prelude::*,
    input::mouse::*,
};
use std::f32::consts::PI;

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl CameraController {
    fn new() -> Self {
        Self { speed: 12.0, sensitivity: 0.2, yaw: 0.0, pitch: 0.0 }
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub controller: CameraController,
}

impl MainCamera {
    pub fn new() -> Self {
        Self { controller: CameraController::new(), }
    }
}

#[derive(Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, 
            camera_control
                .run_if(any_with_component::<PrimaryWindow>)
                .run_if(any_with_component::<MainCamera>)
        );
    }
}

fn grab_mode(grab: &bool) -> CursorGrabMode {
    match grab {
        true => CursorGrabMode::Confined,
        false => CursorGrabMode::None
    }
}

fn camera_control(
    time: Res<Time>,
    mut grabbed: Local<bool>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut window: Single<Mut<Window>, With<PrimaryWindow>>,
    mut cameras: Query<(Mut<MainCamera>, Mut<Transform>)>,
) {
    if kbd.just_pressed(KeyCode::Escape) {
        *grabbed = !*grabbed;
    }

    window.cursor_options.grab_mode = grab_mode(&grabbed);
    window.cursor_options.visible = !*grabbed;

    if !*grabbed { return };

    let delta_time = time.delta().as_secs_f32();

    let mut motion = Vec2::ZERO;
    for event in evr_motion.read() {
        motion -= event.delta;
    }

    if let Ok((mut camera, mut transform)) = cameras.single_mut() {
        let mut speed = camera.controller.speed;
        if kbd.pressed(KeyCode::ControlLeft) { speed *= 2.0 }

        if kbd.pressed(KeyCode::KeyW) {
            let mut forward = transform.forward().as_vec3();
            forward.y = 0.0;

            transform.translation += forward.normalize_or_zero() * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyD) {
            let right = transform.right().normalize();
            transform.translation += right * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyS) {
            let mut back = transform.back().as_vec3();
            back.y = 0.0;

            transform.translation += back.normalize_or_zero() * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyA) {
            let left = transform.left().normalize();
            transform.translation += left * speed * delta_time;
        }

        if kbd.pressed(KeyCode::ShiftLeft) {
            transform.translation.y -= speed * delta_time;
        }
        if kbd.pressed(KeyCode::Space) {
            transform.translation.y += speed * delta_time;
        }

        // Rotate camera
        let contr = &mut camera.controller;
        contr.yaw += motion.x.to_radians() * contr.sensitivity;
        contr.pitch += motion.y.to_radians() * contr.sensitivity;
        contr.pitch = contr.pitch.clamp(-PI/2.02, PI/2.02);
        
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            camera.controller.yaw,
            camera.controller.pitch,
            0.0
        );
    }
}
