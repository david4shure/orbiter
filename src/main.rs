use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use sphere_camera::SphericalCameraPlugin;

mod sphere_camera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_earth)
        .add_plugins((WorldInspectorPlugin::new(),SphericalCameraPlugin))
        .run();
}

fn setup(mut commands: Commands, ass: Res<AssetServer>) {
    let earth_handle = ass.load("earth_hd_rotated.glb#Scene0");
    let skybox_handle = ass.load("skybox2.glb#Scene0");

    // add earth
    commands.spawn((
        SceneBundle {
            scene: earth_handle,
            transform: Transform::from_xyz(0., 0., 0.),
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

    commands.spawn(
sphere_camera::SphereCamera {
            radius: 350.,
            ..Default::default()
        },
    );

    commands.spawn(
        Camera3dBundle {
            transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        }
    );
}

fn rotate_earth(mut query: Query<&mut Transform, With<sphere_camera::EarthBody>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(time.delta_seconds() * 0.4);
    }
}