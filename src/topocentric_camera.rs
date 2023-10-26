use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::sphere_camera;
pub struct TopoCentricCameraPlugin;

#[derive(Default, Reflect, Component, Resource)]
#[reflect(Component)]
pub struct AltitudeAzimuthCamera {
    pub altitude: f32,
    pub azimuth: f32,
    pub roll: f32,
}

impl Plugin for TopoCentricCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, topo_free_look)
           .add_systems(Update, sync_topo_free_look)
           .register_type::<AltitudeAzimuthCamera>();
    }
}

pub fn topo_free_look(
    mut altaz: Query<&mut AltitudeAzimuthCamera>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut scale = 1.5;

    if keys.pressed(KeyCode::ShiftLeft) {
        scale = 0.379;
    }

    let mut altaz_in = altaz.get_single_mut().unwrap();

    if keys.pressed(KeyCode::A) {
        altaz_in.azimuth -= time.delta_seconds() * scale;
    }
    if keys.pressed(KeyCode::D) {
        altaz_in.azimuth += time.delta_seconds() * scale;
    }

    if keys.pressed(KeyCode::W) {
        altaz_in.altitude += time.delta_seconds() * scale;
    }
    if keys.pressed(KeyCode::S) {
        altaz_in.altitude -= time.delta_seconds() * scale;
    }

    if keys.pressed(KeyCode::Q) {
        altaz_in.roll += time.delta_seconds() * scale;
    }
    if keys.pressed(KeyCode::E) {
        altaz_in.roll -= time.delta_seconds() * scale;
    }
}

pub fn sync_topo_free_look(
    mut camera_trans_query: Query<&mut Transform, With<Camera3d>>,
    sphere_camera_query: Query<&sphere_camera::SphereCamera>,
    mut altaz: Query<&mut AltitudeAzimuthCamera>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let altaz_in = altaz.get_single_mut().unwrap();

    let sphere_camera = match sphere_camera_query.get_single() {
        Ok(sphere_camera) => sphere_camera,
        Err(_) => return,
    };

    if !sphere_camera.look_outward {
        return;
    }

    let mut camera_transform: Mut<'_, Transform> = match camera_trans_query.get_single_mut() {
        Ok(camera_transform) => camera_transform,
        Err(_) => return,
    };

    camera_transform.rotation = Quat::from_rotation_x(0.);

    camera_transform.rotate_y(-altaz_in.azimuth);
    camera_transform.rotate_local_x(altaz_in.altitude);
    camera_transform.rotate_local_z(altaz_in.roll);
}