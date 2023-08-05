use std::{marker::PhantomData, sync::Arc};

use bevy::prelude::*;
use colliders::PolyanyaCollider;
use parry3d::shape::HeightField;

mod colliders;

pub struct PolyanyaNavMeshPlugin<ColliderComponent> {
    settings: NavMeshSettings,
    _collider_type: PhantomData<ColliderComponent>,
}

// #[derive(Resource)]
// struct PolyNavMesh {
//     heightfield: Arc<HeightField>,
//     // navmesh: NavMesh,
// }

#[derive(Resource)]
struct NavHeightField {
    heightfield: Arc<HeightField>,
}

impl<C> PolyanyaNavMeshPlugin<C>
where
    C: PolyanyaCollider,
{
    pub fn new(settings: NavMeshSettings) -> Self {
        Self {
            settings,
            _collider_type: PhantomData,
        }
    }
}

pub struct NavMeshSettings {

}

impl<C> Plugin for PolyanyaNavMeshPlugin<C>
where
    C: PolyanyaCollider,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, gen_navmesh::<C>);
    }
}

// TODO: Current design only allows for one navmesh.
fn gen_navmesh<C: PolyanyaCollider>(collider_query: Query<(Entity, &C, &GlobalTransform)>, heightfield: Res<NavHeightField>) {
}
