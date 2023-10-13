use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod sphere_camera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Update, rotate_earth)
        .add_systems(Update, sphere_camera::sphere_camera)
        .add_systems(Update, lock_camera_to_rotation)
        .add_systems(Update, sync_base_theta_for_sphere_camera)
        .add_systems(Update, toggle_look_outward_camera)
        .register_type::<sphere_camera::SphereCamera>()
        .run();
}

fn setup(mut commands: Commands, ass: Res<AssetServer>) {
    let earth_handle = ass.load("earth_hd.glb#Scene0");
    let skybox_handle = ass.load("skybox1.glb#Scene0");

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 20., 15.).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        sphere_camera::SphereCamera {
            radius: 350.,
            ..Default::default()
        },
    ));

    // add earth
    commands.spawn((
        SceneBundle {
            scene: earth_handle,
            transform: Transform::from_xyz(0., 0., 0.)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.)),
            ..default()
        },
        sphere_camera::EarthBody,
    ));

    // Skybox
    commands
        .spawn(SceneBundle {
            scene: skybox_handle,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .insert(Name::new("Sky"));
}

fn rotate_earth(mut query: Query<&mut Transform, With<sphere_camera::EarthBody>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(time.delta_seconds() * 0.4);
    }
}

fn sync_base_theta_for_sphere_camera(
    mut earth_trans_q: Query<(&mut sphere_camera::EarthBody, &mut Transform)>,
    mut sphere_cam_q: Query<&mut sphere_camera::SphereCamera>,
    
) {
    for (_, transform) in earth_trans_q.iter_mut() {
        let euler = transform.rotation.to_euler(EulerRot::ZYX);

        for mut pan_orbit in sphere_cam_q.iter_mut() {
            pan_orbit.base_theta = euler.0;     
        }
    }
}

fn toggle_look_outward_camera(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut sphere_camera::SphereCamera>,
) {
    if keys.just_pressed(KeyCode::R) {
        for mut sphere_camera in query.iter_mut() {
            sphere_camera.look_outward = !sphere_camera.look_outward;
        }
    }
}

fn lock_camera_to_rotation(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut sphere_camera::SphereCamera>,
) {
    for mut sphere_camera in query.iter_mut() {
        if keys.just_pressed(KeyCode::L) {
            sphere_camera.locked = !sphere_camera.locked;
        }
    }
}
