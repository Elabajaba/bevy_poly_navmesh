use bevy::prelude::Component;
use parry3d::{
    bounding_volume::Aabb,
    shape::{SharedShape, TypedShape},
};

use crate::PolyanyaCollider;

#[derive(Component)]
pub struct Parry3dCollider {
    pub collider: SharedShape,
}

impl PolyanyaCollider for Parry3dCollider {
    fn into_typed_shape(&self) -> TypedShape {
        self.collider.as_typed_shape()
    }

    fn t_compute_local_aabb(&self) -> Aabb {
        self.collider.compute_local_aabb()
    }
}
