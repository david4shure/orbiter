use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use crate::topocentric_camera;
use crate::orbit;
pub struct SphericalCameraPlugin;

impl Plugin for SphericalCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_sphere_camera_from_mouse_motion)
            .add_systems(Update, sync_sphere_cam_to_3d_cam)
            .add_systems(Update, lock_camera_to_rotation)
            .add_systems(Update, sync_base_theta_for_sphere_camera)
            .add_systems(Update, toggle_look_outward_camera)
            .add_systems(Update, disable_mouse_scroll)
            .register_type::<SphereCamera>();
    }
}

#[derive(Component)]
pub struct FixMarker;

#[derive(Reflect, Component, Resource)]
#[reflect(Component)]
pub struct SphereCamera {
    pub radius: f32,
    pub base_theta: f32,
    pub theta: f32,
    pub phi: f32,
    pub locked: bool,
    pub look_outward: bool,
    pub frozen: bool,
    pub up: Vec3,
    pub min_radius: f32,
}

impl Default for SphereCamera {
    fn default() -> Self {
        SphereCamera {
            base_theta: 0.,
            radius: 3500.0,
            theta: 0.,
            phi: std::f32::consts::PI / 2.,
            locked: false,
            look_outward: false,
            frozen: false,
            up: Vec3::new(0., 1., 0.),
            min_radius: 500.1,
        }
    }
}

pub fn camera_coords_and_look_vector(sphere_camera: &SphereCamera) -> (Vec3, Vec3, Vec3) {
    let mut phi = sphere_camera.phi;
    let theta = sphere_camera.theta;
    let radius = sphere_camera.radius;

    const FLIP_PADDING: f32 = 0.000015;

    if phi >= std::f32::consts::PI - FLIP_PADDING {
        phi = std::f32::consts::PI - FLIP_PADDING;
    } else if phi <= FLIP_PADDING {
        phi = FLIP_PADDING;
    }

    let pos = to_cart_coords(radius, theta, phi);
    let up = to_cart_coords(radius + 10., theta, phi);
    let north = to_cart_coords(radius, theta, phi - 0.001);

    (pos, up, north)
}

pub fn to_cart_coords(r: f32, theta: f32, phi: f32) -> Vec3 {
    let x = r * phi.sin() * theta.cos();
    let y = r * phi.cos();
    let z = r * phi.sin() * theta.sin();

    Vec3::new(x, y, z)
}

pub fn sync_sphere_cam_to_3d_cam(
    mut sphere_camera_query: Query<&mut SphereCamera>,
    mut camera_trans_query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut transform = match camera_trans_query.get_single_mut() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let sphere_camera = match sphere_camera_query.get_single_mut() {
        Ok(sphere_camera) => sphere_camera,
        Err(_) => return,
    };

    if sphere_camera.look_outward {
        return;
    }

    let pos = to_cart_coords(sphere_camera.radius, sphere_camera.theta, sphere_camera.phi);
    let look = to_cart_coords(
        sphere_camera.radius + 10.,
        sphere_camera.theta,
        sphere_camera.phi,
    );

    let mut look_at: Vec3 = Vec3::new(0., 0., 0.);
    if sphere_camera.look_outward {
        look_at = look;
    }

    transform.translation = pos;
    transform.look_at(look_at, Vec3::Y);
    transform.rotate_around(Vec3::new(0., 0., 0.), Quat::from_rotation_x(0.));
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn update_sphere_camera_from_mouse_motion(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut sphere_camera_query: Query<&mut SphereCamera>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    keys: Res<Input<KeyCode>>,
) {
    // change input mapping for orbit and panning here
    let rotate_button = MouseButton::Left;
    let mut scroll = 0.0;
    let mut net_motion: Vec2 = Vec2::ZERO;

    let mut scroll_scale = 50.0;
    let mut rotate_scale = 1.0;

    if keys.pressed(KeyCode::ShiftLeft) {
        scroll_scale = 5.;
        rotate_scale = 0.1;
    }

    if input_mouse.pressed(rotate_button) {
        for ev in ev_motion.read() {
            net_motion += ev.delta;
        }
    }

    for ev in ev_scroll.read() {
        scroll += ev.y;
    }

    let transform = match camera_query.get_single_mut() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let mut sphere_camera = match sphere_camera_query.get_single_mut() {
        Ok(sphere_camera) => sphere_camera,
        Err(_) => return,
    };

    let distance_from_cam_to_body = transform.translation.distance(Vec3::new(0., 0., 0.));

    scroll_scale *= distance_from_cam_to_body / 1000.;

    let mut phi = sphere_camera.phi;
    let mut theta = sphere_camera.theta;
    let mut radius = sphere_camera.radius;

    const FLIP_PADDING: f32 = 0.0015;

    let d_phi = rotate_scale * -net_motion.y / 500.;
    let d_theta = rotate_scale * net_motion.x / 500.;
    let d_radius = -(scroll_scale * scroll);

    phi += d_phi;
    theta += d_theta;
    radius += d_radius;

    if phi >= std::f32::consts::PI - FLIP_PADDING {
        phi = std::f32::consts::PI - FLIP_PADDING;
    } else if phi <= FLIP_PADDING {
        phi = FLIP_PADDING;
    }

    if radius < sphere_camera.min_radius {
        radius = sphere_camera.min_radius;
    }

    sphere_camera.phi = phi;
    sphere_camera.radius = radius;
    sphere_camera.theta = theta;
}

pub fn disable_mouse_scroll(mut sphere_cam_q: Query<&mut SphereCamera>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::F) {
        for mut sphere_camera in sphere_cam_q.iter_mut() {
            sphere_camera.frozen = !sphere_camera.frozen;
        }
    }
}

pub fn sync_base_theta_for_sphere_camera(
    mut earth_trans_q: Query<(&mut orbit::EarthBody, &mut Transform)>,
    mut sphere_cam_q: Query<&mut SphereCamera>,
) {
    for (_, transform) in earth_trans_q.iter_mut() {
        let euler = transform.rotation.to_euler(EulerRot::ZYX);

        for mut pan_orbit in sphere_cam_q.iter_mut() {
            pan_orbit.base_theta = euler.0;
        }
    }
}

pub fn toggle_look_outward_camera(
    keys: Res<Input<KeyCode>>,
    mut sphere_camera_query: Query<&mut SphereCamera>,
    mut camera_entity_query: Query<Entity, With<Camera3d>>,
    mut earth_entity_query: Query<Entity, With<orbit::EarthBody>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fix_marker_query: Query<Entity, With<FixMarker>>,
    ass: Res<AssetServer>,
    mut altaz: Query<&mut topocentric_camera::AltitudeAzimuthCamera>,
) {
    let mut sphere_camera = sphere_camera_query.single_mut();
    let mut altaz_in = altaz.get_single_mut().unwrap();

    if keys.just_pressed(KeyCode::R) && sphere_camera.locked {
        let camera_entity = camera_entity_query.get_single_mut().unwrap();
        let earth_entity = earth_entity_query.get_single_mut().unwrap();

        altaz_in.altitude = 0.;
        altaz_in.azimuth = 0.;
        altaz_in.roll = 0.;

        sphere_camera.look_outward = !sphere_camera.look_outward;

        if sphere_camera.look_outward {
            sphere_camera.radius = sphere_camera.min_radius;

            let (pos, up, north) = camera_coords_and_look_vector(&sphere_camera);

            commands.entity(camera_entity).despawn();
            let new_camera = commands
                .spawn(Camera3dBundle {
                    transform: Transform::from_xyz(0., 0., 0.).looking_at(-Vec3::Z, Vec3::Y),
                    ..default()
                })
                .id();

            let trans: Transform = Transform::IDENTITY
                .with_translation(pos)
                .looking_at(north, up);
            let cube = commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            radius: 3.0,
                            rings: 10 as usize,
                            depth: 3.,
                            ..default()
                        })),
                        material: materials.add(Color::rgb(1., 1., 1.).into()),
                        transform: trans,
                        ..default()
                    },
                    FixMarker,
                ))
                .id();

            let north_handle: Handle<Scene> = ass.load("north.glb#Scene0");
            let south_handle: Handle<Scene> = ass.load("south.glb#Scene0");
            let east_handle: Handle<Scene> = ass.load("east.glb#Scene0");
            let west_handle: Handle<Scene> = ass.load("west.glb#Scene0");

            let north = commands
                .spawn(SceneBundle {
                    scene: north_handle,
                    transform: Transform::from_xyz(0.0, -0.53, -30.0),
                    ..Default::default()
                })
                .id();

            let south = commands
                .spawn(SceneBundle {
                    scene: south_handle,
                    transform: Transform::from_xyz(0.0, -0.53, 30.0)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                    ..Default::default()
                })
                .id();

            let east = commands
                .spawn(SceneBundle {
                    scene: east_handle,
                    transform: Transform::from_xyz(30.0, -0.53, 0.0)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                    ..Default::default()
                })
                .id();

            let west = commands
                .spawn(SceneBundle {
                    scene: west_handle,
                    transform: Transform::from_xyz(-30.0, -0.53, 0.0),
                    ..Default::default()
                })
                .id();

            commands.entity(earth_entity).add_child(cube);
            commands.entity(cube).add_child(new_camera);
            commands.entity(cube).add_child(north);
            commands.entity(cube).add_child(south);
            commands.entity(cube).add_child(east);
            commands.entity(cube).add_child(west);

        } else {
            let fix_marker_cube = fix_marker_query.get_single_mut().unwrap();

            commands.entity(fix_marker_cube).despawn_recursive();

            let new_camera = commands
                .spawn(Camera3dBundle {
                    transform: Transform::from_xyz(0., 0., 0.).looking_at(Vec3::ZERO, Vec3::Y),
                    ..default()
                })
                .id();

            commands.entity(earth_entity).add_child(new_camera);
        }
    }
}

pub fn lock_camera_to_rotation(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut SphereCamera>,
    mut camera_query: Query<Entity, With<Camera3d>>,
    mut earth_query: Query<Entity, With<orbit::EarthBody>>,
    mut commands: Commands,
) {
    let camera_entity = camera_query.get_single_mut().unwrap();
    let earth_entity = earth_query.get_single_mut().unwrap();

    for mut sphere_camera in query.iter_mut() {
        if keys.just_pressed(KeyCode::L) && !sphere_camera.look_outward {

            if sphere_camera.locked {
                // locked -> unlocked
                println!("locked -> unlocked");
                commands
                    .entity(earth_entity)
                    .remove_children(&[camera_entity]);
                commands.entity(camera_entity).despawn();
                commands.spawn(Camera3dBundle {
                    transform: Transform::from_xyz(0., 20., 50.).looking_at(Vec3::ZERO, Vec3::Y),
                    ..default()
                });
            } else {
                // unlocked -> locked
                println!("unlocked -> locked");
                commands.entity(camera_entity).despawn();
                let new_camera = commands
                    .spawn(Camera3dBundle {
                        transform: Transform::from_xyz(0., 20., 44.)
                            .looking_at(Vec3::ZERO, Vec3::Y),
                        ..default()
                    })
                    .id();
                commands.entity(earth_entity).add_child(new_camera);
            }

            sphere_camera.locked = !sphere_camera.locked;
        }
    }
}