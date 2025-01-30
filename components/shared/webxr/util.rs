use crate::FrameUpdateEvent;
use crate::HitTestId;
use crate::HitTestSource;
use euclid::Transform3D;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct ClipPlanes {
    pub near: f32,
    pub far: f32,
    /// Was there an update that needs propagation to the client?
    update: bool,
}

impl Default for ClipPlanes {
    fn default() -> Self {
        ClipPlanes {
            near: 0.1,
            far: 1000.,
            update: false,
        }
    }
}

impl ClipPlanes {
    pub fn update(&mut self, near: f32, far: f32) {
        self.near = near;
        self.far = far;
        self.update = true;
    }

    /// Checks for and clears the pending update flag
    pub fn recently_updated(&mut self) -> bool {
        if self.update {
            self.update = false;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
/// Holds on to hit tests
pub struct HitTestList {
    tests: Vec<HitTestSource>,
    uncommitted_tests: Vec<HitTestSource>,
}

impl HitTestList {
    pub fn request_hit_test(&mut self, source: HitTestSource) {
        self.uncommitted_tests.push(source)
    }

    pub fn commit_tests(&mut self) -> Vec<FrameUpdateEvent> {
        let mut events = vec![];
        for test in self.uncommitted_tests.drain(..) {
            events.push(FrameUpdateEvent::HitTestSourceAdded(test.id));
            self.tests.push(test);
        }
        events
    }

    pub fn tests(&self) -> &[HitTestSource] {
        &self.tests
    }

    pub fn cancel_hit_test(&mut self, id: HitTestId) {
        self.tests.retain(|s| s.id != id);
        self.uncommitted_tests.retain(|s| s.id != id);
    }
}

#[inline]
/// Construct a projection matrix given the four angles from the center for the faces of the viewing frustum
pub fn fov_to_projection_matrix<T, U>(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    clip_planes: ClipPlanes,
) -> Transform3D<f32, T, U> {
    let near = clip_planes.near;
    // XXXManishearth deal with infinite planes
    let left = left.tan() * near;
    let right = right.tan() * near;
    let top = top.tan() * near;
    let bottom = bottom.tan() * near;

    frustum_to_projection_matrix(left, right, top, bottom, clip_planes)
}

#[inline]
/// Construct matrix given the actual extent of the viewing frustum on the near plane
pub fn frustum_to_projection_matrix<T, U>(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    clip_planes: ClipPlanes,
) -> Transform3D<f32, T, U> {
    let near = clip_planes.near;
    let far = clip_planes.far;

    let w = right - left;
    let h = top - bottom;
    let d = far - near;

    // Column-major order
    Transform3D::new(
        2. * near / w,
        0.,
        0.,
        0.,
        0.,
        2. * near / h,
        0.,
        0.,
        (right + left) / w,
        (top + bottom) / h,
        -(far + near) / d,
        -1.,
        0.,
        0.,
        -2. * far * near / d,
        0.,
    )
}
