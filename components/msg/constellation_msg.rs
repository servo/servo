/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps
//! reduce coupling between these two components.

use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use hyper::header::Headers;
use hyper::method::Method;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender, IpcSharedMemory};
use layers::geometry::DevicePixel;
use rand::{OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;
use util::geometry::{PagePx, ViewportPx};
use webdriver_msg::{LoadStatus, WebDriverScriptCommand};
use webrender_traits;

#[derive(Deserialize, Serialize)]
pub struct ConstellationChan<T: Deserialize + Serialize>(pub IpcSender<T>);

impl<T: Deserialize + Serialize> ConstellationChan<T> {
    pub fn new() -> (IpcReceiver<T>, ConstellationChan<T>) {
        let (chan, port) = ipc::channel().unwrap();
        (port, ConstellationChan(chan))
    }
}

impl<T: Serialize + Deserialize> Clone for ConstellationChan<T> {
    fn clone(&self) -> ConstellationChan<T> {
        ConstellationChan(self.0.clone())
    }
}

pub type PanicMsg = (Option<PipelineId>, String);

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
    flags KeyModifiers: u8 {
        const NONE = 0x00,
        const SHIFT = 0x01,
        const CONTROL = 0x02,
        const ALT = 0x04,
        const SUPER = 0x08,
    }
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    LoadUrl(PipelineId, LoadData, IpcSender<LoadStatus>),
    Refresh(PipelineId, IpcSender<LoadStatus>),
    ScriptCommand(PipelineId, WebDriverScriptCommand),
    SendKeys(PipelineId, Vec<(Key, KeyModifiers, KeyState)>),
    TakeScreenshot(PipelineId, IpcSender<Option<Image>>),
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize, HeapSizeOf)]
pub enum PixelFormat {
    K8,         // Luminance channel only
    KA8,        // Luminance + alpha
    RGB8,       // RGB, 8 bits per channel
    RGBA8,      // RGB + alpha, 8 bits per channel
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize, HeapSizeOf)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
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
}

impl LoadData {
    pub fn new(url: Url) -> LoadData {
        LoadData {
            url: url,
            method: Method::Get,
            headers: Headers::new(),
            data: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize)]
pub enum NavigationDirection {
    Forward,
    Back,
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize)]
pub struct FrameId(pub u32);

/// Each pipeline ID needs to be unique and unguessable to prevent spoofing.
/// It also needs to be possible to generate the pipeline ID from an iframe
/// element (this simplifies a lot of other code that makes use of pipeline IDs).
///
/// To achieve this, each pipeline index is an opaque 128 bit random token.
/// This allows pipeline IDs to be generated by any thread without conflicting
/// with pipeline IDs created by other script threads or the constellation.
///
/// It's best to think of the pipeline ID as a simple unique identifier
/// that doesn't convey any more information.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct PipelineId {
    buf: [u8; 16],
}

impl PipelineId {
    pub fn new() -> PipelineId {
        let mut rng = OsRng::new().unwrap();
        let mut id = PipelineId {
          buf: [0; 16],
        };
        rng.fill_bytes(&mut id.buf);
        id
    }
}

impl fmt::Display for PipelineId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for &byte in self.buf.iter() {
            try!(write!(fmt, "{:x} ", byte))
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct SubpageId(pub u32);

impl From<webrender_traits::PipelineId> for PipelineId {
    fn from(wr_pipeline_id: webrender_traits::PipelineId) -> Self {
        PipelineId { buf: wr_pipeline_id.buf }
    }
}

impl From<PipelineId> for webrender_traits::PipelineId {
    fn from(pipeline_id: PipelineId) -> Self {
        webrender_traits::PipelineId { buf: pipeline_id.buf }
    }
}
