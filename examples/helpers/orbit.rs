//! An orbit controls plugin for bevy.
//!
//! To control the camera, drag the mouse. The left button rotates. The wheel
//! zooms.
//!
//! ## Usage
//!
//! Register the [`OrbitCameraPlugin`], and insert the [`OrbitCamera`] struct
//! into the entity containing the camera.
//!
//! For example, within the startup system:
//!
//! ```no_compile
//! commands
//!     .spawn_bundle(PerspectiveCameraBundle {
//!         transform: Transform::from_translation(Vec3::new(-3.0, 3.0, 5.0))
//!             .looking_at(Vec3::default(), Vec3::Y),
//!         ..Default::default()
//!     })
//!     .insert(OrbitCamera::default());
//! ```
//!
//! ## Compatibility
//!
//! - `v2.x` – Bevy `0.5`.
//! - `v1.x` – Bevy `0.4`.

use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseScrollUnit::{Line, Pixel};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::{Camera, Projection};
use bevy::window::PrimaryWindow;

const LINE_TO_PIXEL_RATIO: f32 = 0.1;

#[derive(Event)]
pub enum CameraEvents {
    Orbit(Vec2),
    Pan(Vec2),
    Zoom(f32),
}

#[derive(Component)]
pub struct OrbitCamera {
    pub x_angle: f32,
    pub y_angle: f32,
    pub min_y_angle: f32,
    pub max_y_angle: f32,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub center: Vec3,
    pub max_center: Vec3,
    pub min_center: Vec3,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub enabled: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            x_angle: 45.0_f32.to_radians(),
            y_angle: 45.0_f32.to_radians(),
            max_y_angle: 85.0_f32.to_radians(),
            min_y_angle: 1.0_f32.to_radians(),
            distance: 5.0,
            min_distance: 1.0,
            max_distance: 400.0,
            center: Vec3::ZERO,
            max_center: Vec3::splat(100.0),
            min_center: Vec3::splat(-100.0),
            rotate_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.8,
            enabled: true,
        }
    }
}

pub struct OrbitCameraPlugin;
impl OrbitCameraPlugin {
    pub fn update_transform_system(
        mut query: Query<(&OrbitCamera, &mut Transform), (Changed<OrbitCamera>, With<Camera>)>,
    ) {
        for (camera, mut transform) in query.iter_mut() {
            if camera.enabled {
                let rot =
                    Quat::from_rotation_z(camera.x_angle) * Quat::from_rotation_x(camera.y_angle);
                transform.translation = (rot * (Vec3::Z)) * camera.distance + camera.center;
                transform.look_at(camera.center, Vec3::Z);
            }
        }
    }

    pub fn emit_motion_events(
        mut events: EventWriter<CameraEvents>,
        mut mouse_motion_events: EventReader<MouseMotion>,
        mouse_button_input: Res<Input<MouseButton>>,
        mut query: Query<&OrbitCamera>,
    ) {
        let mut delta = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            delta += event.delta;
        }
        for camera in query.iter_mut() {
            if camera.enabled {
                if mouse_button_input.pressed(MouseButton::Right) {
                    events.send(CameraEvents::Orbit(delta))
                }

                if mouse_button_input.pressed(MouseButton::Left) {
                    events.send(CameraEvents::Pan(delta))
                }
            }
        }
    }

    pub fn mouse_motion_system(
        time: Res<Time>,
        primary_window: Query<&Window, With<PrimaryWindow>>,
        mut events: EventReader<CameraEvents>,
        mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera, &Projection)>,
    ) {
        for (mut camera, transform, _, projection) in query.iter_mut() {
            if !camera.enabled {
                continue;
            }
            let perspective_proj = match projection {
                Projection::Perspective(perspective_proj) => perspective_proj,
                Projection::Orthographic(_) => {
                    panic!("Orbit camera does not support orthographic perspective")
                }
            };

            for event in events.iter() {
                match event {
                    CameraEvents::Orbit(delta) => {
                        //continue;
                        camera.x_angle -=
                            delta.x * camera.rotate_sensitivity * time.delta_seconds();
                        camera.y_angle -=
                            delta.y * camera.rotate_sensitivity * time.delta_seconds();
                        camera.y_angle = camera
                            .y_angle
                            .min(camera.max_y_angle)
                            .max(camera.min_y_angle);
                    }
                    CameraEvents::Pan(delta) => {
                        // make panning distance independent of resolution and FOV,
                        let window = Vec2::new(
                            primary_window.get_single().unwrap().width() as f32,
                            primary_window.get_single().unwrap().height() as f32,
                        );
                        let mut delta_scaled = delta.clone();
                        delta_scaled *= Vec2::new(
                            perspective_proj.fov * perspective_proj.aspect_ratio,
                            perspective_proj.fov,
                        ) / window;

                        // Transform to local axes
                        let mut right = transform.rotation * Vec3::X;
                        let mut up = transform.rotation * Vec3::Y;
                        // Remove Z component and normalize in order to make pan speed independent from incline
                        right.z = 0.0;
                        up.z = 0.0;
                        right = right.normalize_or_zero() * -delta_scaled.x;
                        up = up.normalize_or_zero() * delta_scaled.y;

                        // make panning proportional to distance away from focus point
                        let pan_vector = Mat3::from_cols(Vec3::X, Vec3::Y, Vec3::ZERO)
                            * ((right + up) * camera.distance)
                            * camera.pan_sensitivity;
                        camera.center += pan_vector;

                        camera.center = camera.center.max(camera.min_center).min(camera.max_center);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn emit_zoom_events(
        mut events: EventWriter<CameraEvents>,
        mut mouse_wheel_events: EventReader<MouseWheel>,
        mut query: Query<&OrbitCamera>,
    ) {
        let mut total = 0.0;
        for event in mouse_wheel_events.iter() {
            total += event.y
                * match event.unit {
                    Line => 1.0,
                    Pixel => LINE_TO_PIXEL_RATIO,
                };
        }

        if total != 0.0 {
            for camera in query.iter_mut() {
                if camera.enabled {
                    events.send(CameraEvents::Zoom(total));
                }
            }
        }
    }

    pub fn zoom_system(
        mut query: Query<&mut OrbitCamera, With<Camera>>,
        mut events: EventReader<CameraEvents>,
    ) {
        for mut camera in query.iter_mut() {
            for event in events.iter() {
                if camera.enabled {
                    if let CameraEvents::Zoom(distance) = event {
                        camera.distance *= camera.zoom_sensitivity.powf(*distance);
                        camera.distance = camera
                            .distance
                            .min(camera.max_distance)
                            .max(camera.min_distance);
                    }
                }
            }
        }
    }
}
impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::emit_motion_events)
            .add_systems(Update, Self::mouse_motion_system)
            .add_systems(Update, Self::emit_zoom_events)
            .add_systems(Update, Self::zoom_system)
            .add_systems(Update, Self::update_transform_system)
            // .register_inspectable::<OrbitCamera>()
            .add_event::<CameraEvents>();
    }
}
