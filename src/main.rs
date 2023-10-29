use bevy::prelude::*;

use bevy::pbr::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use orbit::OrbitPlugin;
use sphere_camera::SphericalCameraPlugin;
use topocentric_camera::TopoCentricCameraPlugin;

mod lines;
mod orbit;
mod sphere_camera;
mod topocentric_camera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(TopoCentricCameraPlugin)
        .add_plugins((WorldInspectorPlugin::new(), SphericalCameraPlugin))
        .add_plugins(OrbitPlugin)
        .run();
}

fn setup(mut commands: Commands, ass: Res<AssetServer>) {
    let earth_handle = ass.load("earth.glb#Scene0");
    let skybox_handle = ass.load("sky_actual_constel.glb#Scene0");
    let moon_handle = ass.load("moon.glb#Scene0");

    // Earth
    commands.spawn((
        SceneBundle {
            scene: earth_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        orbit::EarthBody,
    ));

    // Sphere Camera
    commands.spawn(sphere_camera::SphereCamera {
        radius: 600.,
        ..Default::default()
    });

    // Altitude Azimuth Camera Controls
    commands.spawn(topocentric_camera::AltitudeAzimuthCamera {
        altitude: 0.,
        azimuth: 0.,
        roll: 0.,
    });

    // 3D Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::Y, Vec3::Y),
            ..default()
        },
        NotShadowCaster,
    ));

    //commands.insert_resource(AmbientLight {
    //    color: Color::WHITE,
    //    brightness: 0.1,
    //});

    // Sun light
    //commands
    //    .spawn(DirectionalLightBundle {
    //        directional_light: DirectionalLight {
    //            color: Color::WHITE,
    //            illuminance: 100000.,
    //            shadows_enabled: true,
    //            ..default()
    //        },
    //        transform: Transform::from_xyz(1000.0, -100.0, 0.0)
    //            .with_rotation(Quat::from_rotation_x(23.4 * std::f32::consts::PI / 180.)),
    //        ..default()
    //    })
    //    .insert(Name::new("Sun Light"));

    commands.spawn(PointLightBundle {
        // transform: Transform::from_xyz(5.0, 8.0, 2.0),
        transform: Transform::from_xyz(5000.0, 4.0, 0.0),
        point_light: PointLight {
            range: 100000.,
            intensity: 10000000000., // lumens - roughly a 100W non-halogen incandescent bulb
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Skybox
    commands
        .spawn((
            SceneBundle {
                scene: skybox_handle,
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            },
            NotShadowCaster,
        ))
        .insert(Name::new("Sky"));

    // Moon
    commands
        .spawn((
            SceneBundle {
                scene: moon_handle.clone(),
                transform: Transform::from_xyz(0.0, orbit::REAL_TO_WORLD * 384472.282, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.)),
                ..Default::default()
            },
            orbit::CelestialBody {
                name: "Moon".to_string(),
                focus_idx: 1,
                viewport_position: None,
            },
            orbit::MoonBody,
        ))
        .insert(Name::new("Moon"));
}

//fn rotate_earth(mut query: Query<&mut Transform, With<orbit::EarthBody>>, time: Res<Time>) {
//    for mut transform in query.iter_mut() {
//        transform.rotate_y(time.delta_seconds() * 0.4);
//    }
//}
