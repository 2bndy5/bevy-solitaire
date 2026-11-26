//! The 3D scene: a tilted camera looking down at a felt table, lit from above.
//!
//! Game logic and layout work in 2D "table" coordinates (see [`crate::layout`]): `x` to the
//! right and `y` away from the player. In the world, the table is the XZ plane with `Y` up, so a
//! table point `(x, y)` is the world point `(x, height, -y)`.
use std::f32::consts::FRAC_PI_6;

use bevy::{
    light::CascadeShadowConfigBuilder, prelude::*, render::view::Tonemapping, window::PrimaryWindow,
};

use crate::{layout::Layout, state::GameState};

/// The color of the felt on the card table.
pub const FELT: Color = Color::srgb(0.11, 0.42, 0.23);
/// The camera's vertical field of view. Narrow, to keep perspective distortion gentle.
const CAMERA_FOV: f32 = FRAC_PI_6;
/// Brightness of the light shining on the table.
///
/// Chosen with the camera's default exposure so a white card face lit by both lights comes out at
/// about full white, without tone mapping washing the colors out.
const SUN_LUX: f32 = 2200.0;
/// Brightness of the ambient light, which keeps shadows from going black.
const AMBIENT_BRIGHTNESS: f32 = 300.0;

/// Converts a table point at `height` above the felt to a world position.
pub fn to_world(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, -point.y)
}

/// Converts a world position to a table point (dropping its height).
pub fn to_table(world: Vec3) -> Vec2 {
    Vec2::new(world.x, -world.z)
}

/// Marks the scene's camera.
#[derive(Component)]
pub struct TableCamera;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(FELT))
            .insert_resource(GlobalAmbientLight {
                brightness: AMBIENT_BRIGHTNESS,
                ..default()
            })
            .add_systems(Startup, setup_scene)
            .init_resource::<Layout>()
            .add_systems(Update, fit_to_window);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    commands.spawn((
        TableCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV,
            near: 10.0,
            ..default()
        }),
        camera_transform(1000.0, Layout::default().camera_tilt),
        // Show the card art's colors as they are.
        Tonemapping::None,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: SUN_LUX,
            shadow_maps_enabled: true,
            ..default()
        },
        // Light from above, slightly from the top left, so cards cast short shadows.
        Transform::from_xyz(-250.0, 1000.0, -350.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 4000.0,
            ..default()
        }
        .build(),
    ));
    // The felt extends well past the screen edges, so no background shows around it.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(4000.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: FELT,
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));
    game_state.set(GameState::Menu);
}

/// The camera's transform when it is `distance` away from the table's center, leaning back by
/// `tilt` radians.
fn camera_transform(distance: f32, tilt: f32) -> Transform {
    // Lean back toward the player (+Z), keeping the far side of the table at the top of the
    // screen.
    let eye = Vec3::new(0.0, tilt.cos(), tilt.sin()) * distance;
    Transform::from_translation(eye).looking_at(Vec3::ZERO, Vec3::NEG_Z)
}

/// Picks the table layout for the window's shape, and moves the camera so the whole table fits.
fn fit_to_window(
    window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut layout: ResMut<Layout>,
    mut camera: Single<&mut Transform, With<TableCamera>>,
) {
    let size = window.size();
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let new_layout = Layout::for_window(size);
    if *layout != new_layout {
        *layout = new_layout;
    }
    let aspect = size.x / size.y;
    let half_fov = (CAMERA_FOV / 2.0).tan();
    let half = layout.size / 2.0;
    let distance = (half.y / half_fov).max(half.x / (half_fov * aspect));
    **camera = camera_transform(distance, layout.camera_tilt);
}

/// Where the pointer at `viewport_pos` meets a horizontal plane `height` above the table, in table
/// coordinates.
pub fn pointer_on_plane(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    viewport_pos: Vec2,
    height: f32,
) -> Option<Vec2> {
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let distance = ray.intersect_plane(Vec3::Y * height, InfinitePlane3d::new(Vec3::Y))?;
    Some(to_table(ray.get_point(distance)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_world_round_trip() {
        let point = Vec2::new(12.0, -34.0);
        let world = to_world(point, 5.0);
        assert_eq!(world, Vec3::new(12.0, 5.0, 34.0));
        assert_eq!(to_table(world), point);
    }

    #[test]
    fn camera_keeps_far_side_at_top() {
        let transform = camera_transform(1000.0, 0.3);
        // The camera's "up" points away from the player (toward -Z, i.e. +y on the table).
        assert!(transform.up().z < 0.0);
        assert!(transform.translation.y > 0.0 && transform.translation.z > 0.0);
    }
}
