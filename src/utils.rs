use bevy::{
    prelude::{Component, GlobalTransform, Vec2},
    utils::{HashMap, HashSet, Instant},
};
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::PolyanyaCollider;

#[derive(Component)]
pub struct NavMeshAffector;

pub fn into_polyanya_mesh(
    cdt: &ConstrainedDelaunayTriangulation<Point2<f32>>,
    // navigable_faces: &HashSet<usize>,
    // obstacle_edges: &HashSet<usize>,
) -> polyanya::Mesh {
    todo!()
    // TODO: Converting a polyanya TriMesh to a Polyanya Mesh is slow, so we should just directly create a polyanya mesh.
    let now = Instant::now();
    let mut vertices: Vec<Vec2> = Vec::new();
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    let mut vert_indices: HashMap<usize, usize> = HashMap::new();
    let navigable_faces = compute_navigable_faces(&cdt);
    for face in cdt.inner_faces() {
        if !navigable_faces.contains(&face.index()) {
            continue;
        }
        let mut indices = [0; 3];
        face.vertices().iter().enumerate().for_each(|(i, vertex)| {
            vertex.index();
            let temp_pos = vertex.position();
            let pos = Vec2::new(temp_pos.x, temp_pos.y);
            let idx = *vert_indices.entry(vertex.index()).or_insert_with(|| {
                let idx = vertices.len();
                vertices.push(pos);
                idx
            });
            // if let Some(index) = vertices.iter().position(|v| *v == pos) {
            //     idx = index;
            // } else {
            //     vertices.push(pos);
            // }
            indices[i] = idx;
        });
        triangles.push(indices);
    }
    let elapsed = now.elapsed();
    println!("into_polyanya_mesh 1 took: {:?}", elapsed);
    let mesh = polyanya::Trimesh {
        vertices,
        triangles,
    };
    let ret = mesh.into();
    let elapsed = now.elapsed();
    println!("into_polyanya_mesh total took: {:?}", elapsed);
    ret
}

pub fn compute_navigable_faces<C: PolyanyaCollider>(
    cdt: &ConstrainedDelaunayTriangulation<Point2<f32>>,
    colliders: (&C, &GlobalTransform),
    // obstacle_edges: &HashSet<usize>,
) -> HashSet<usize> {
    let now = Instant::now();
    // let mut obstacle_edges = HashSet::new();
    // for edge in cdt.directed_edges() {
    //     if edge.is_constraint_edge() {
    //         // println!("{} is constraint edge", edge.index());
    //         obstacle_edges.insert(edge.index());
    //     }
    // }

    let mut navigable_faces = HashSet::new();

    for face in cdt.inner_faces() {
        let mut touches_constraint = 0;
        for edge in face.adjacent_edges() {
            // if obstacle_edges.contains(&edge.index()) {
            //     touches_constraint = true;
            //     break;
            // }
            if edge.is_constraint_edge() {
                touches_constraint += 1;
            }
            // if outgoing_obstacle != incoming_obstacle {
            //     if incoming_obstacle {
            //         navigable_faces.insert(face.index());
            //     }
            //     continue 'face_loop;
            // }
        }

        if touches_constraint < 2 {
            navigable_faces.insert(face.index());
        }

        // for edge in edge.ccw_iter().skip(1) {
        //     let outgoing_obstacle = obstacle_edges.contains(&edge.fix());
        //     let incoming_obstacle = obstacle_edges.contains(&edge.sym().fix());
        //     if outgoing_obstacle != incoming_obstacle {
        //         if incoming_obstacle {
        //             navigable_faces.insert(face.fix());
        //         }
        //         continue 'face_loop;
        //     }
        // }
    }
    let elapsed = now.elapsed();
    // println!("compute_navigable_faces took: {:?}", elapsed);
    navigable_faces
}

// fn compute_navigable_faces(
//     &self,
//     triangulation: &Triangulation,
//     obstacle_edges: &HashSet<usize>,
//   ) -> HashSet<usize> {
//     let mut navigable_faces = HashSet::new();
//     'face_loop: for face in triangulation.triangles() {
//       if let Some(edge) = face.adjacent_edge() {
//         for edge in edge.ccw_iter().skip(1) {
//           let outgoing_obstacle = obstacle_edges.contains(&edge.fix());
//           let incoming_obstacle = obstacle_edges.contains(&edge.sym().fix());
//           if outgoing_obstacle != incoming_obstacle {
//             if incoming_obstacle {
//               navigable_faces.insert(face.fix());
//             }
//             continue 'face_loop;
//           }
//         }
//       }
//     }
//     navigable_faces
//   }
