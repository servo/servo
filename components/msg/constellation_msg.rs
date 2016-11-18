/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps
//! reduce coupling between these two components.

use std::cell::Cell;
use std::fmt;
use uuid::{Uuid, NAMESPACE_URL};
use servo_rand;
use servo_rand::Rand;
use webrender_traits;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum KeyState {
    Pressed,
    Released,
    Repeated,
}

//N.B. Based on the glutin key enum
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum Key {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,

    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,

    NavigateBackward,
    NavigateForward,
}

bitflags! {
    #[derive(Deserialize, Serialize)]
    pub flags KeyModifiers: u8 {
        const NONE = 0x00,
        const SHIFT = 0x01,
        const CONTROL = 0x02,
        const ALT = 0x04,
        const SUPER = 0x08,
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize)]
pub enum TraversalDirection {
    Forward(usize),
    Back(usize),
}

/// Each pipeline ID needs to be unique and unguessable to prevent spoofing.
/// It also needs to be possible to generate the pipeline ID from an iframe
/// element (this simplifies a lot of other code that makes use of pipeline IDs).
///
/// To achieve this, each pipeline ID is a v4 UUID, which is an opaque 128 bit random token.
/// They are generated via a cryptographically-secure RNG, which provides the key properties
/// of uniqueness and unguessability that together make pipeline IDs unforgeable.
/// Uniqueness can be provided by any RNG with a sufficiently large period,
/// which will prevent the same pipeline ID from being generated more than once.
/// However, in order for the RNG output to be unguessable (i.e. an attacker
/// cannot predict the next output based on previous outputs, or reconstruct
/// previous outputs if (part of) the internal RNG state is revealed), the RNG
/// must be cryptographically secure.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct PipelineId {
    #[ignore_heap_size_of = "no heapsize for uuid"]
    pub id: Uuid,
}

impl PipelineId {
    pub fn new() -> PipelineId {
        let mut rng = servo_rand::thread_rng();
        PipelineId {
            id: Uuid::rand(&mut rng),
        }
    }

    pub fn to_webrender(&self) -> webrender_traits::PipelineId {
        webrender_traits::PipelineId(self.id)
    }
}

impl fmt::Display for PipelineId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.id.simple().fmt(fmt)
    }
}

thread_local!(pub static TOP_LEVEL_FRAME_ID: Cell<Option<FrameId>> = Cell::new(None));

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct FrameId {
    #[ignore_heap_size_of = "no heapsize for uuid"]
    pub id: Uuid,
}

impl FrameId {
    pub fn new() -> FrameId {
        let mut rng = servo_rand::thread_rng();
        FrameId {
            id: Uuid::rand(&mut rng),
        }
    }

    /// Each script and layout thread should have the top-level frame id installed,
    /// since it is used by crash reporting.
    pub fn install(id: FrameId) {
        TOP_LEVEL_FRAME_ID.with(|tls| tls.set(Some(id)))
    }

    pub fn installed() -> Option<FrameId> {
        TOP_LEVEL_FRAME_ID.with(|tls| tls.get())
    }
}

impl fmt::Display for FrameId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.id.simple().fmt(fmt)
    }
}

// We provide ids just for unit testing.
pub const TEST_PIPELINE_ID: PipelineId = PipelineId { id: NAMESPACE_URL };
pub const TEST_FRAME_ID: FrameId = FrameId { id: NAMESPACE_URL };

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub enum FrameType {
    IFrame,
    MozBrowserIFrame,
}
