use core::panic;
use std::{marker::PhantomData, sync::Arc};

use bevy::prelude::*;
use bevy_pathmesh::PathMesh;
pub use colliders::PolyanyaCollider;
use parry3d::shape::{HeightField, SharedShape, TypedShape};
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use utils::{compute_navigable_faces, NavMeshAffector};

use crate::utils::into_polyanya_mesh;

pub mod colliders;
pub mod utils;

pub struct PolyanyaNavMeshPlugin<ColliderComponent> {
    _settings: NavMeshSettings,
    _collider_type: PhantomData<ColliderComponent>,
}

impl<C> Plugin for PolyanyaNavMeshPlugin<C>
where
    C: PolyanyaCollider,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(DrawCdt(true))
            .add_systems(PostStartup, gen_navmesh::<C>)
            .add_systems(Update, draw_cdt::<C>)
            .add_systems(PreUpdate, update_navmesh::<C>);
    }
}

pub struct NavMeshSettings {}

#[derive(Component)]
pub struct MyCollider {
    collider: SharedShape,
}

impl PolyanyaCollider for MyCollider {
    fn into_typed_shape(&self) -> parry3d::shape::TypedShape {
        self.collider.as_typed_shape()
    }

    fn t_compute_local_aabb(&self) -> parry3d::bounding_volume::Aabb {
        self.collider.compute_local_aabb()
    }
}

#[derive(Resource)]
pub struct PolyNavMesh {
    cdt: ConstrainedDelaunayTriangulation<Point2<f32>>,
    pub navmesh_handle: Handle<PathMesh>,
    pub dimensions: (f32, f32),
}

#[derive(Resource)]
pub struct NavHeightField {
    pub heightfield: Arc<HeightField>,
}

impl<C> PolyanyaNavMeshPlugin<C>
where
    C: PolyanyaCollider,
{
    pub fn new(settings: NavMeshSettings) -> Self {
        Self {
            _settings: settings,
            _collider_type: PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct DrawCdt(pub bool);

fn draw_cdt<C: PolyanyaCollider>(
    draw_cdt: Res<DrawCdt>,
    cdt: Res<PolyNavMesh>,
    mut gizmos: Gizmos,
    collider_query: Query<(&C, &GlobalTransform), With<NavMeshAffector>>,
) {
    if draw_cdt.0 {
        let navigable_faces = compute_navigable_faces(&cdt.cdt);
        for face in cdt.cdt.inner_faces() {
            if navigable_faces.contains(&face.index()) {
                let vertices = face.vertices();
                let a = vertices[0].position();
                let b = vertices[1].position();
                let c = vertices[2].position();
                let a = Vec3::new(a.x, 2.0, a.y);
                let b = Vec3::new(b.x, 2.0, b.y);
                let c = Vec3::new(c.x, 2.0, c.y);
                gizmos.line(a, b, Color::GREEN);
                gizmos.line(b, c, Color::GREEN);
                gizmos.line(c, a, Color::GREEN);
            } else {
                let vertices = face.vertices();
                let a = vertices[0].position();
                let b = vertices[1].position();
                let c = vertices[2].position();
                let a = Vec3::new(a.x, 2.1, a.y);
                let b = Vec3::new(b.x, 2.1, b.y);
                let c = Vec3::new(c.x, 2.1, c.y);
                gizmos.line(a, b, Color::RED);
                gizmos.line(b, c, Color::RED);
                gizmos.line(c, a, Color::RED);
            }
        }

        // for edge in cdt.cdt.undirected_edges() {
        //     let vertices = edge.vertices();
        //     let start_pos = vertices[0].position();
        //     let end_pos = vertices[1].position();
        //     // if let (Some(start_pos), Some(end_pos)) =
        //     //     (vertices[0].position(), vertices[1].position())
        //     // {
        //     let colour = if edge.() {
        //         Color::RED
        //     } else {
        //         Color::GREEN
        //     };
        //     // let colour = if edge.is_constraint_edge() {
        //     //     Color::RED
        //     // } else {
        //     //     Color::GREEN
        //     // };
        //     let start = Vec3::new(start_pos.x, 2.0, start_pos.y);
        //     let end = Vec3::new(end_pos.x, 2.0, end_pos.y);
        //     gizmos.line(start, end, colour);
        //     // }
        // }
    }
}

// TODO: Current design only allows for one navmesh.
fn gen_navmesh<C: PolyanyaCollider>(
    mut commands: Commands,
    collider_query: Query<(Entity, &C, &GlobalTransform), With<NavMeshAffector>>,
    heightfield: Res<NavHeightField>,
    mut navmeshes: ResMut<Assets<PathMesh>>,
) {
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<_>>::new();

    // Insert the outer corners of the heightfield into the cdt.
    let scale = heightfield.heightfield.scale();
    let hscale = scale * 0.5;

    println!("hscale: {:?}", hscale);
    cdt.insert(Point2::new(-hscale.x, -hscale.z))
        .expect("failed to insert vertex into cdt");
    cdt.insert(Point2::new(-hscale.x, hscale.z))
        .expect("failed to insert vertex into cdt");
    cdt.insert(Point2::new(hscale.x, hscale.z))
        .expect("failed to insert vertex into cdt");
    cdt.insert(Point2::new(hscale.x, -hscale.z))
        .expect("failed to insert vertex into cdt");
    // Insert the outer vertices of the heightfield into the cdt.

    // TODO: Use height differences in heightfield to determine if something is walkable or not.
    // let scale = heightfield.heightfield.scale();
    // for height in heightfield.heightfield.heights() {
    //     let height = height * scale.y;
    //     let point = Point2::new(height.x, height.z);
    //     cdt.insert(point).expect("failed to insert vertex into cdt");
    // }
    // for tri in heightfield.heightfield.triangles() {
    //     temp.insert(tri.a);
    //     // cdt.insert(Point2::new(tri.a.x, tri.a.z))
    //     //     .expect("failed to insert vertex into cdt");
    //     // cdt.insert(Point2::new(tri.b.x, tri.b.z))
    //     //     .expect("failed to insert vertex into cdt");
    //     // cdt.insert(Point2::new(tri.c.x, tri.c.z))
    //     //     .expect("failed to insert vertex into cdt");
    // }

    for (_entity, collider, transform) in collider_query.iter() {
        let shape = collider.into_typed_shape();
        add_collider_to_navmesh(shape, transform, &mut cdt);
    }

    let navmesh = into_polyanya_mesh(&cdt);
    let pathmesh = PathMesh::from_polyanya_mesh(navmesh);
    let navmesh_handle = navmeshes.add(pathmesh);

    commands.insert_resource(PolyNavMesh {
        cdt,
        navmesh_handle,
        dimensions: (scale.x, scale.z),
    });
}

fn update_navmesh<C: PolyanyaCollider>(
    // mut commands: Commands,
    collider_query: Query<(Entity, &C, &GlobalTransform), (With<NavMeshAffector>, Added<C>)>,
    mut navmesh: ResMut<PolyNavMesh>,
    mut navmeshes: ResMut<Assets<PathMesh>>,
) {
    let mut has_changed = false;
    let cdt = &mut navmesh.cdt;
    for (_entity, collider, transform) in collider_query.iter() {
        has_changed = true;
        let shape = collider.into_typed_shape();
        add_collider_to_navmesh(shape, transform, cdt);
    }
    if has_changed {
        println!("updating navmesh");
        let mesh = into_polyanya_mesh(cdt);
        let pathmesh = PathMesh::from_polyanya_mesh(mesh);
        let current_handle = navmesh.navmesh_handle.clone();
        let navmesh_handle = navmeshes.add(pathmesh);
        navmeshes.remove(current_handle);
        navmesh.navmesh_handle = navmesh_handle;
    }
}

fn add_collider_to_navmesh(
    shape: TypedShape,
    transform: &GlobalTransform,
    cdt: &mut ConstrainedDelaunayTriangulation<Point2<f32>>,
) {
    let points = handle_shape(shape, transform);
    let mut handles = Vec::with_capacity(points.len());
    for point in points {
        handles.push(cdt.insert(point).expect("failed to insert vertex into cdt"));
    }
    for i in 0..handles.len() {
        let j = (i + 1) % handles.len();
        cdt.add_constraint(handles[i], handles[j]);
    }
}

fn handle_shape(
    shape: TypedShape,
    transform: &GlobalTransform,
    // cdt: &mut ConstrainedDelaunayTriangulation<Point2<f32>>,
) -> Vec<Point2<f32>> {
    match shape {
        TypedShape::Cuboid(cube) => {
            // Use the half_extents for cubes
            let a = transform.transform_point(Vec3::new(
                -cube.half_extents.x,
                0.0,
                -cube.half_extents.z,
            ));
            let b = transform.transform_point(Vec3::new(
                -cube.half_extents.x,
                0.0,
                cube.half_extents.z,
            ));
            let c =
                transform.transform_point(Vec3::new(cube.half_extents.x, 0.0, cube.half_extents.z));
            let d = transform.transform_point(Vec3::new(
                cube.half_extents.x,
                0.0,
                -cube.half_extents.z,
            ));

            vec![
                Point2::new(a.x, a.z),
                Point2::new(b.x, b.z),
                Point2::new(c.x, c.z),
                Point2::new(d.x, d.z),
            ]
        }
        TypedShape::Compound(_compound) => {
            // We need to boolean the shapes together to get the outer points.
            // for shape in compound.shapes() {
            // }
            // // We only want the outer points of the compound shape, otherwise this won't work.
            // let mut points = Vec::new();
            // for shape in compound.shapes() {
            //     let mut shape_points = handle_shape(shape.1.as_typed_shape(), transform);

            //     points.append(&mut shape_points);
            // }

            // compound.aabbs()

            // points
            todo!()
        }
        TypedShape::Ball(_) => todo!(),
        TypedShape::Capsule(_) => todo!(),
        TypedShape::Segment(_) => todo!(),
        TypedShape::Triangle(_) => todo!(),
        TypedShape::TriMesh(_) => todo!(),
        TypedShape::ConvexPolyhedron(_) => todo!(),
        TypedShape::Cylinder(_) => todo!(),
        TypedShape::Cone(_) => todo!(),
        TypedShape::RoundCuboid(_) => todo!(),
        TypedShape::RoundTriangle(_) => todo!(),
        TypedShape::RoundCylinder(_) => todo!(),
        TypedShape::RoundCone(_) => todo!(),
        TypedShape::RoundConvexPolyhedron(_) => todo!(),
        TypedShape::Polyline(_) => panic!("Cannot use Polyline as a navmesh affector."),
        TypedShape::Custom(_) => panic!("Cannot use Custom as a navmesh affector."),
        TypedShape::HalfSpace(_) => panic!("Cannot use HalfSpace as a navmesh affector."),
        TypedShape::HeightField(_) => panic!("Cannot use HeightField as a navmesh affector."),
    }
}
