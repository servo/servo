/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps
//! reduce coupling between these two components.

use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use hyper::header::Headers;
use hyper::method::Method;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use layers::geometry::DevicePixel;
use rand::Rand;
use rand::os::OsRng;
use std::fmt;
use url::Url;
use util::geometry::{PagePx, ViewportPx};
use uuid::Uuid;
use webdriver_msg::{LoadStatus, WebDriverScriptCommand};
use webrender_traits;

pub type PanicMsg = (Option<PipelineId>, String, String);

#[derive(Copy, Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// http://www.w3.org/TR/css-device-adapt/#initial-viewport
    pub initial_viewport: TypedSize2D<ViewportPx, f32>,

    /// The "viewing area" in page px. See `PagePx` documentation for details.
    pub visible_viewport: TypedSize2D<PagePx, f32>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>,
}

#[derive(Deserialize, Eq, PartialEq, Serialize, Copy, Clone, HeapSizeOf)]
pub enum WindowSizeType {
    Initial,
    Resize,
}

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

#[derive(Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    GetWindowSize(PipelineId, IpcSender<WindowSizeData>),
    LoadUrl(PipelineId, LoadData, IpcSender<LoadStatus>),
    Refresh(PipelineId, IpcSender<LoadStatus>),
    ScriptCommand(PipelineId, WebDriverScriptCommand),
    SendKeys(PipelineId, Vec<(Key, KeyModifiers, KeyState)>),
    SetWindowSize(PipelineId, Size2D<u32>, IpcSender<WindowSizeData>),
    TakeScreenshot(PipelineId, IpcSender<Option<Image>>),
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize, HeapSizeOf)]
pub enum PixelFormat {
    K8,         // Luminance channel only
    KA8,        // Luminance + alpha
    RGB8,       // RGB, 8 bits per channel
    RGBA8,      // RGB + alpha, 8 bits per channel
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    pub bytes: IpcSharedMemory,
    #[ignore_heap_size_of = "Defined in webrender_traits"]
    pub id: Option<webrender_traits::ImageKey>,
}

/// Similar to net::resource_thread::LoadData
/// can be passed to LoadUrl to load a page with GET/POST
/// parameters or headers
#[derive(Clone, Deserialize, Serialize)]
pub struct LoadData {
    pub url: Url,
    pub method: Method,
    pub headers: Headers,
    pub data: Option<Vec<u8>>,
    pub referrer_policy: Option<ReferrerPolicy>,
    pub referrer_url: Option<Url>,
}

impl LoadData {
    pub fn new(url: Url, referrer_policy: Option<ReferrerPolicy>, referrer_url: Option<Url>) -> LoadData {
        LoadData {
            url: url,
            method: Method::Get,
            headers: Headers::new(),
            data: None,
            referrer_policy: referrer_policy,
            referrer_url: referrer_url,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize)]
pub enum NavigationDirection {
    Forward(usize),
    Back(usize),
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize)]
pub struct FrameId(pub u32);

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
///
/// These properties mean that pipeline IDs enact a capability security scheme,
/// where the only way to interact with a pipeline is to be given its pipeline ID.
/// These properties also allow pipeline IDs to be generated by any thread without
/// conflicting with pipeline IDs created by other script threads or the constellation.
///
/// It's best to think of the pipeline ID as a simple unique identifier
/// that identifies a pipeline (and allows interaction),
/// but doesn't convey any additional information.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct PipelineId {
    #[ignore_heap_size_of = "no heapsize impl for uuid"]
    pub id: Uuid,
}



impl PipelineId {
    pub fn new() -> PipelineId {
        // TODO(aneeshusa): cache this RNG for speed
        // Since iframes create their own PipelineIds,
        // this needs to be cached in the script thread.
        let mut rng = OsRng::new().unwrap();
        PipelineId {
            id: Uuid::rand(&mut rng),
        }
    }

    pub fn to_webrender(&self) -> webrender_traits::PipelineId {
        webrender_traits::PipelineId { id: self.id }
    }
}

impl fmt::Display for PipelineId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.id.simple().fmt(fmt)
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct SubpageId(pub u32);

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub enum FrameType {
    IFrame,
    MozBrowserIFrame,
}

/// [Policies](https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-states)
/// for providing a referrer header for a request
#[derive(Clone, Copy, Debug, Deserialize, HeapSizeOf, Serialize)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoRefWhenDowngrade,
    OriginOnly,
    OriginWhenCrossOrigin,
    UnsafeUrl,
}
