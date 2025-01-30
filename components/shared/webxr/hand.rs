use crate::Native;
use euclid::RigidTransform3D;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct HandSpace;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct Hand<J> {
    pub wrist: Option<J>,
    pub thumb_metacarpal: Option<J>,
    pub thumb_phalanx_proximal: Option<J>,
    pub thumb_phalanx_distal: Option<J>,
    pub thumb_phalanx_tip: Option<J>,
    pub index: Finger<J>,
    pub middle: Finger<J>,
    pub ring: Finger<J>,
    pub little: Finger<J>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct Finger<J> {
    pub metacarpal: Option<J>,
    pub phalanx_proximal: Option<J>,
    pub phalanx_intermediate: Option<J>,
    pub phalanx_distal: Option<J>,
    pub phalanx_tip: Option<J>,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct JointFrame {
    pub pose: RigidTransform3D<f32, HandSpace, Native>,
    pub radius: f32,
}

impl Default for JointFrame {
    fn default() -> Self {
        Self {
            pose: RigidTransform3D::identity(),
            radius: 0.,
        }
    }
}

impl<J> Hand<J> {
    pub fn map<R>(&self, map: impl (Fn(&Option<J>, Joint) -> Option<R>) + Copy) -> Hand<R> {
        Hand {
            wrist: map(&self.wrist, Joint::Wrist),
            thumb_metacarpal: map(&self.thumb_metacarpal, Joint::ThumbMetacarpal),
            thumb_phalanx_proximal: map(&self.thumb_phalanx_proximal, Joint::ThumbPhalanxProximal),
            thumb_phalanx_distal: map(&self.thumb_phalanx_distal, Joint::ThumbPhalanxDistal),
            thumb_phalanx_tip: map(&self.thumb_phalanx_tip, Joint::ThumbPhalanxTip),
            index: self.index.map(|f, j| map(f, Joint::Index(j))),
            middle: self.middle.map(|f, j| map(f, Joint::Middle(j))),
            ring: self.ring.map(|f, j| map(f, Joint::Ring(j))),
            little: self.little.map(|f, j| map(f, Joint::Little(j))),
        }
    }

    pub fn get(&self, joint: Joint) -> Option<&J> {
        match joint {
            Joint::Wrist => self.wrist.as_ref(),
            Joint::ThumbMetacarpal => self.thumb_metacarpal.as_ref(),
            Joint::ThumbPhalanxProximal => self.thumb_phalanx_proximal.as_ref(),
            Joint::ThumbPhalanxDistal => self.thumb_phalanx_distal.as_ref(),
            Joint::ThumbPhalanxTip => self.thumb_phalanx_tip.as_ref(),
            Joint::Index(f) => self.index.get(f),
            Joint::Middle(f) => self.middle.get(f),
            Joint::Ring(f) => self.ring.get(f),
            Joint::Little(f) => self.little.get(f),
        }
    }
}

impl<J> Finger<J> {
    pub fn map<R>(&self, map: impl (Fn(&Option<J>, FingerJoint) -> Option<R>) + Copy) -> Finger<R> {
        Finger {
            metacarpal: map(&self.metacarpal, FingerJoint::Metacarpal),
            phalanx_proximal: map(&self.phalanx_proximal, FingerJoint::PhalanxProximal),
            phalanx_intermediate: map(&self.phalanx_intermediate, FingerJoint::PhalanxIntermediate),
            phalanx_distal: map(&self.phalanx_distal, FingerJoint::PhalanxDistal),
            phalanx_tip: map(&self.phalanx_tip, FingerJoint::PhalanxTip),
        }
    }

    pub fn get(&self, joint: FingerJoint) -> Option<&J> {
        match joint {
            FingerJoint::Metacarpal => self.metacarpal.as_ref(),
            FingerJoint::PhalanxProximal => self.phalanx_proximal.as_ref(),
            FingerJoint::PhalanxIntermediate => self.phalanx_intermediate.as_ref(),
            FingerJoint::PhalanxDistal => self.phalanx_distal.as_ref(),
            FingerJoint::PhalanxTip => self.phalanx_tip.as_ref(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum FingerJoint {
    Metacarpal,
    PhalanxProximal,
    PhalanxIntermediate,
    PhalanxDistal,
    PhalanxTip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum Joint {
    Wrist,
    ThumbMetacarpal,
    ThumbPhalanxProximal,
    ThumbPhalanxDistal,
    ThumbPhalanxTip,
    Index(FingerJoint),
    Middle(FingerJoint),
    Ring(FingerJoint),
    Little(FingerJoint),
}
