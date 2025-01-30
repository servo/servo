use crate::ApiSpace;
use crate::Native;
use crate::Space;
use euclid::Point3D;
use euclid::RigidTransform3D;
use euclid::Rotation3D;
use euclid::Vector3D;
use std::f32::EPSILON;
use std::iter::FromIterator;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// https://immersive-web.github.io/hit-test/#xrray
pub struct Ray<Space> {
    /// The origin of the ray
    pub origin: Vector3D<f32, Space>,
    /// The direction of the ray. Must be normalized.
    pub direction: Vector3D<f32, Space>,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// https://immersive-web.github.io/hit-test/#enumdef-xrhittesttrackabletype
pub enum EntityType {
    Point,
    Plane,
    Mesh,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// https://immersive-web.github.io/hit-test/#dictdef-xrhittestoptionsinit
pub struct HitTestSource {
    pub id: HitTestId,
    pub space: Space,
    pub ray: Ray<ApiSpace>,
    pub types: EntityTypes,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct HitTestId(pub u32);

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// Vec<EntityType>, but better
pub struct EntityTypes {
    pub point: bool,
    pub plane: bool,
    pub mesh: bool,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct HitTestResult {
    pub id: HitTestId,
    pub space: RigidTransform3D<f32, HitTestSpace, Native>,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// The coordinate space of a hit test result
pub struct HitTestSpace;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct Triangle {
    pub first: Point3D<f32, Native>,
    pub second: Point3D<f32, Native>,
    pub third: Point3D<f32, Native>,
}

impl EntityTypes {
    pub fn is_type(self, ty: EntityType) -> bool {
        match ty {
            EntityType::Point => self.point,
            EntityType::Plane => self.plane,
            EntityType::Mesh => self.mesh,
        }
    }

    pub fn add_type(&mut self, ty: EntityType) {
        match ty {
            EntityType::Point => self.point = true,
            EntityType::Plane => self.plane = true,
            EntityType::Mesh => self.mesh = true,
        }
    }
}

impl FromIterator<EntityType> for EntityTypes {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = EntityType>,
    {
        iter.into_iter().fold(Default::default(), |mut acc, e| {
            acc.add_type(e);
            acc
        })
    }
}

impl Triangle {
    /// https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    pub fn intersect(
        self,
        ray: Ray<Native>,
    ) -> Option<RigidTransform3D<f32, HitTestSpace, Native>> {
        let Triangle {
            first: v0,
            second: v1,
            third: v2,
        } = self;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);
        if a > -EPSILON && a < EPSILON {
            // ray is parallel to triangle
            return None;
        }

        let f = 1. / a;

        let s = ray.origin - v0.to_vector();

        // barycentric coordinate of intersection point u
        let u = f * s.dot(h);
        // barycentric coordinates have range (0, 1)
        if u < 0. || u > 1. {
            // the intersection is outside the triangle
            return None;
        }

        let q = s.cross(edge1);
        // barycentric coordinate of intersection point v
        let v = f * ray.direction.dot(q);

        // barycentric coordinates have range (0, 1)
        // and their sum must not be greater than 1
        if v < 0. || u + v > 1. {
            // the intersection is outside the triangle
            return None;
        }

        let t = f * edge2.dot(q);

        if t > EPSILON {
            let origin = ray.origin + ray.direction * t;

            // this is not part of the Möller-Trumbore algorithm, the hit test spec
            // requires it has an orientation such that the Y axis points along
            // the triangle normal
            let normal = edge1.cross(edge2).normalize();
            let y = Vector3D::new(0., 1., 0.);
            let dot = normal.dot(y);
            let rotation = if dot > -EPSILON && dot < EPSILON {
                // vectors are parallel, return the vector itself
                // XXXManishearth it's possible for the vectors to be
                // antiparallel, unclear if normals need to be flipped
                Rotation3D::identity()
            } else {
                let axis = normal.cross(y);
                let cos = normal.dot(y);
                // This is Rotation3D::around_axis(axis.normalize(), theta), however
                // that is just Rotation3D::quaternion(axis.normalize().xyz * sin, cos),
                // which is Rotation3D::quaternion(cross, dot)
                Rotation3D::quaternion(axis.x, axis.y, axis.z, cos)
            };

            return Some(RigidTransform3D::new(rotation, origin));
        }

        // triangle is behind ray
        None
    }
}
