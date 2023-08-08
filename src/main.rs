use std::sync::Arc;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_poly_navmesh::{
    colliders::parry3d_collider::Parry3dCollider, utils::NavMeshAffector, DrawCdt, NavHeightField,
    NavMeshSettings, PolyanyaNavMeshPlugin,
};
use camera_controller::{CameraController, CameraControllerPlugin};
use parry3d::{
    na::{DMatrix, Vector3},
    shape::{HeightField, SharedShape},
};

mod camera_controller;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PolyanyaNavMeshPlugin::<Parry3dCollider>::new(NavMeshSettings {}),
            CameraControllerPlugin,
        ))
        .insert_resource(DrawCdt(true))
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_heightfield, add_obstacle))
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(60.0, 50.0, 50.0)
                .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
            ..default()
        },
        CameraController::default(),
    ));

    let heightfield_heights: Vec<f32> = (0..(50 * 50))
        .map(|value| {
            let position = value / 50;

            (position as f32 / 10.0).sin() / 10.0
        })
        .collect();
    let heights = DMatrix::from_vec(50, 50, heightfield_heights);

    let heightfield = HeightField::new(heights, Vector3::new(50.0, 50.0, 50.0));

    commands.insert_resource(NavHeightField {
        heightfield: Arc::new(heightfield),
    });
}

fn draw_heightfield(heightfield: Res<NavHeightField>, mut gizmos: Gizmos, draw_cdt: Res<DrawCdt>) {
    if !draw_cdt.0 {
        for tri in heightfield.heightfield.triangles() {
            let vertices = tri.vertices();
            let a = vertices[0].coords;
            let b = vertices[1].coords;
            let c = vertices[2].coords;
            let a = Vec3::new(a.x, a.y, a.z);
            let b = Vec3::new(b.x, b.y, b.z);
            let c = Vec3::new(c.x, c.y, c.z);
            gizmos.line(a, b, Color::RED);
            gizmos.line(a, c, Color::RED);
            gizmos.line(b, c, Color::RED);
        }
    }
}

const HALF_WIDTH: f32 = 1.0;
const HALF_HEIGHT: f32 = 1.0;

pub fn add_obstacle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<Input<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    if input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_q.single();
        let window = primary_window.single();
        if let Some(point) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| {
                if let Some(intersection) = ray.intersect_plane(Vec3::new(0.0, 0.5, 0.0), Vec3::Y) {
                    Some(ray.get_point(intersection))
                } else {
                    None
                }
            })
            .flatten()
        {
            // obstacles.obstacles.push(Obstacle::Closed {
            //     // CCW winding order for
            //     vertices: vec![
            //         Vec2::new(point.x - HALF_WIDTH, point.z - HALF_HEIGHT),
            //         Vec2::new(point.x + HALF_WIDTH, point.z - HALF_HEIGHT),
            //         Vec2::new(point.x + HALF_WIDTH, point.z + HALF_HEIGHT),
            //         Vec2::new(point.x - HALF_WIDTH, point.z + HALF_HEIGHT),
            //     ],
            // });

            let mesh = Mesh::from(shape::Cube {
                size: HALF_HEIGHT * 2.0,
            });
            let material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material,
                    transform: Transform::from_translation(Vec3::new(point.x, 0.0, point.z)),
                    ..Default::default()
                },
                Parry3dCollider {
                    collider: SharedShape::cuboid(HALF_WIDTH, 5.0, HALF_HEIGHT),
                },
                NavMeshAffector,
            ));
        }
    }
}
