use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use sphere_camera::SphericalCameraPlugin;
use topocentric_camera::TopoCentricCameraPlugin;

mod sphere_camera;
mod topocentric_camera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_earth)
        .add_plugins(TopoCentricCameraPlugin)
        .add_plugins((WorldInspectorPlugin::new(),SphericalCameraPlugin))
        .run();
}

fn setup(
    mut commands: Commands, 
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let earth_handle = ass.load("earth.glb#Scene0");
    let skybox_handle = ass.load("sky_with_constellations.glb#Scene0");

    // add earth
    commands.spawn((SceneBundle {
        scene: earth_handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    },sphere_camera::EarthBody));

    // Skybox
    commands.spawn(
sphere_camera::SphereCamera {
            radius: 600.,
            ..Default::default()
        },
    );

    commands.spawn(
        Camera3dBundle {
            transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::Y, Vec3::Y),
            ..default()
        }
    );

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 100000.,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(1000.0, -100.0, 0.0),
        ..default()
    }).insert(Name::new("Sun Light"));

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
        transform.rotate_y(time.delta_seconds() * 0.4);
    }
}
