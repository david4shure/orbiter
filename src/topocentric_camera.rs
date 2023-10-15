use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::sphere_camera;
pub struct TopoCentricCameraPlugin;

impl Plugin for TopoCentricCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, topo_free_look);
    }
}

pub fn topo_free_look(
    mut camera_trans_query: Query<&mut Transform, With<Camera3d>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut ev_motion: EventReader<MouseMotion>,
    sphere_camera_query: Query<&sphere_camera::SphereCamera>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {

    let sphere_camera = match sphere_camera_query.get_single() {
        Ok(sphere_camera) => sphere_camera,
        Err(_) => return,
    };

    if !sphere_camera.look_outward {
        return;
    }

    //let mut window = q_windows.single_mut();

    //window.cursor.visible = false;

    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        println!("Cursor is inside the primary window, at {:?}", position);
    } else {
        println!("Cursor is not in the game window.");
    }

    let mut camera_transform: Mut<'_, Transform> = match camera_trans_query.get_single_mut() {
        Ok(camera_transform) => camera_transform,
        Err(_) => return,
    };

    let mut net_motion: Vec2 = Vec2::ZERO;

    for ev in ev_motion.iter() {
        net_motion += ev.delta;
    }

    let mut scale = 1.5;

    if keys.pressed(KeyCode::ShiftLeft) {
        scale = 0.3;
    }

    camera_transform.rotate_local_y(-net_motion.x / 200.);
    camera_transform.rotate_local_x(-net_motion.y / 200.);

    if keys.pressed(KeyCode::Q) {
        camera_transform.rotate_local_z(time.delta_seconds() * scale);
    }
    if keys.pressed(KeyCode::E) {
        camera_transform.rotate_local_z(-time.delta_seconds() * scale);
    }

}
