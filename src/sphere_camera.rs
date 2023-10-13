use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

#[derive(Component)]
pub struct EarthBody;

#[derive(Reflect, Component, Resource)]
#[reflect(Component)]
pub struct SphereCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub body_idx: i32,
    pub radius: f32,
    pub base_theta: f32,
    pub theta: f32,
    pub phi: f32,
    pub locked: bool,
    pub look_outward: bool,
}

impl Default for SphereCamera {
    fn default() -> Self {
        SphereCamera {
            base_theta: 0.,
            body_idx: 0,
            radius: 3500.0,
            theta: 0.,
            phi: 0.,
            locked: false,
            look_outward: false,
        }
    }
}

pub fn to_cart_coords(r: f32, theta: f32, phi: f32) -> (f32, f32, f32) {
    let x = r * phi.sin() * theta.cos();
    let y = r * phi.cos();
    let z = r * phi.sin() * theta.sin();

    (x, y, z)
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn sphere_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut set: ParamSet<(
        Query<(&mut SphereCamera, &mut Transform)>,
        Query<(&EarthBody, &mut Transform)>,
    )>,
    keys: Res<Input<KeyCode>>,
) {
    // change input mapping for orbit and panning here
    let rotate_button = MouseButton::Left;
    let mut scroll = 0.0;
    let mut net_motion: Vec2 = Vec2::ZERO;

    let mut scroll_scale = 50.0;
    let mut rotate_scale = 1.0;

    if keys.pressed(KeyCode::ShiftLeft) {
        scroll_scale = 1.;
        rotate_scale = 0.1;
    }

    if input_mouse.pressed(rotate_button) {
        for ev in ev_motion.iter() {
            net_motion += ev.delta;
        }
    }

    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    for (mut pan_orbit, mut transform) in set.p0().iter_mut() {
        let distance_from_cam_to_body = transform.translation.distance(Vec3::new(0.,0.,0.));

        scroll_scale *= distance_from_cam_to_body / 1000.;

        let mut phi = pan_orbit.phi;
        let mut theta = pan_orbit.theta;
        let mut radius = pan_orbit.radius;

        const FLIP_PADDING: f32 = 0.0015;

        let d_phi = rotate_scale * -net_motion.y / 500.;
        let d_theta = rotate_scale * net_motion.x / 500.;
        let d_radius = -(scroll_scale * scroll);

        if phi >= std::f32::consts::PI - FLIP_PADDING {
            phi = std::f32::consts::PI - FLIP_PADDING;
        } else if phi <= FLIP_PADDING {
            phi = FLIP_PADDING;
        }

        phi += d_phi;
        theta += d_theta;
        radius += d_radius;

        let mut theta_actual = theta;

        if pan_orbit.locked {
            theta_actual -= pan_orbit.base_theta;
        }

        let (x,y,z) = to_cart_coords(radius, theta_actual, phi); 

        let (l_x, l_y, l_z) = to_cart_coords(radius + 10., theta_actual, phi+std::f32::consts::PI/2.);
        

        let mut look_at : Vec3 = Vec3::new(0.,0.,0.);
        if pan_orbit.look_outward {
            look_at = Vec3::new(l_x,l_y,l_z);
        }

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(look_at, Vec3::Y);
        transform.rotate_around(
            Vec3::new(0.,0.,0.),
            Quat::from_rotation_x(std::f32::consts::PI / 2.),
        );

        pan_orbit.phi = phi;
        pan_orbit.radius = radius;
        pan_orbit.theta = theta;
    }
}
