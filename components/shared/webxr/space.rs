use crate::InputId;
use crate::Joint;
use euclid::RigidTransform3D;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// A stand-in type for "the space isn't statically known since
/// it comes from client side code"
pub struct ApiSpace;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum BaseSpace {
    Local,
    Floor,
    Viewer,
    BoundedFloor,
    TargetRay(InputId),
    Grip(InputId),
    Joint(InputId, Joint),
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct Space {
    pub base: BaseSpace,
    pub offset: RigidTransform3D<f32, ApiSpace, ApiSpace>,
}
