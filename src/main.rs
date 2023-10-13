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
        .add_systems(Update, disable_mouse_scroll)
        .register_type::<sphere_camera::SphereCamera>()
        .run();
}

fn setup(mut commands: Commands, ass: Res<AssetServer>) {
    let earth_handle = ass.load("earth_hd_rotated.glb#Scene0");
    let skybox_handle = ass.load("skybox1.glb#Scene0");

    // camera
    // commands.spawn((
    //    Camera3dBundle {
    //        transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::ZERO, Vec3::Y),
    //        ..default()
    //    },
    //    sphere_camera::SphereCamera {
    //        radius: 350.,
    //        ..Default::default()
    //    },
    // ));

    // add earth
    commands.spawn((
        SceneBundle {
            scene: earth_handle,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        sphere_camera::EarthBody,
    )).with_children(|parent| {
        parent.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 20., 44.).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            sphere_camera::SphereCamera {
                radius: 350.,
                ..Default::default()
            }
        ));
    });

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

fn disable_mouse_scroll(
    mut sphere_cam_q: Query<&mut sphere_camera::SphereCamera>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::M) {
        for mut sphere_camera in sphere_cam_q.iter_mut() {
            sphere_camera.frozen = !sphere_camera.frozen;
        }
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
   mut cam_q: Query<(&mut Camera3d, &mut Transform)>,
) {
   if keys.just_pressed(KeyCode::R) {
       for mut sphere_camera in query.iter_mut() {
            for (_,mut trans) in cam_q.iter_mut() {
                sphere_camera.look_outward = !sphere_camera.look_outward;
                let mut theta_actual = sphere_camera.theta;
                if sphere_camera.locked {
                    theta_actual -= sphere_camera.base_theta;
                }
                if sphere_camera.look_outward {
                    // let (l_x, l_y, l_z) = sphere_camera::to_cart_coords(sphere_camera.radius+100., theta_actual, sphere_camera.phi);
                    // trans.look_at(Vec3::new(l_x,l_y,l_z),Vec3::Y);
                    // println!("Toggle Look Outward look_at({},{},{})",l_x,l_y,l_z);
                } else {
                    // let look_at : Vec3 = Vec3::new(0.,0.,0.);
                    // trans.look_at(look_at,Vec3::Y);
                    // println!("Toggle Look Outward look_at({},{},{})",0.,0.,0.);
                }
            }
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
