
use bevy::camera::*;
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
pub use wgpu;


#[derive(Component)]
struct Ground;


#[derive(Component)]
pub struct OrbitCameraSettings {
    pub sensitivity: f32,
    pub target: Vec3,
    pub enabled_button: MouseButton,
}

impl Default for OrbitCameraSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.005,
            target: Vec3::ZERO,
            enabled_button: MouseButton::Right,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCameraSettings::default(),
    ));
}



pub fn set_camera(mut commands: Commands){

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

}


fn orbit_camera_system(
    mut motion_messages: MessageReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut q_camera: Query<(&mut Transform, &OrbitCameraSettings), With<Camera3d>>,
) {
    let mut delta = Vec2::ZERO;
    for message in motion_messages.read() {
        delta += message.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    for (mut transform, settings) in &mut q_camera {
        if !mouse_button.pressed(settings.enabled_button) {
            continue;
        }

        transform.rotate_around(
            settings.target,
            Quat::from_rotation_y(-delta.x * settings.sensitivity),
        );

        let right = transform.right();
        transform.rotate_around(
            settings.target,
            Quat::from_axis_angle(*right, -delta.y * settings.sensitivity),
        );
    }
}


fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        // Calculate if and where the ray is hitting the ground plane.
        && let Some(point) = ray.plane_intersection_point(ground.translation(), InfinitePlane3d::new(ground.up()))
    {
        // Draw a circle just above the ground plane at that position.
        gizmos.circle(
            Isometry3d::new(
                point + ground.up() * 0.01,
                Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
            ),
            0.2,
            Color::WHITE,
        );
    }
}