/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps
//! reduce coupling between these two components.

use geom::rect::Rect;
use geom::size::TypedSize2D;
use geom::scale_factor::ScaleFactor;
use hyper::header::Headers;
use hyper::method::{Method, Get};
use layers::geometry::DevicePixel;
use servo_util::geometry::{PagePx, ViewportPx};
use std::comm::{channel, Sender, Receiver};
use url::Url;

#[deriving(Clone)]
pub struct ConstellationChan(pub Sender<Msg>);

impl ConstellationChan {
    pub fn new() -> (Receiver<Msg>, ConstellationChan) {
        let (chan, port) = channel();
        (port, ConstellationChan(chan))
    }
}

#[deriving(PartialEq)]
pub enum IFrameSandboxState {
    IFrameSandboxed,
    IFrameUnsandboxed
}

// We pass this info to various tasks, so it lives in a separate, cloneable struct.
#[deriving(Clone)]
pub struct Failure {
    pub pipeline_id: PipelineId,
    pub subpage_id: Option<SubpageId>,
}

pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// http://www.w3.org/TR/css-device-adapt/#initial-viewport
    pub initial_viewport: TypedSize2D<ViewportPx, f32>,

    /// The "viewing area" in page px. See `PagePx` documentation for details.
    pub visible_viewport: TypedSize2D<PagePx, f32>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>,
}

#[deriving(PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
    Repeated,
}

//N.B. Straight up copied from glfw-rs
#[deriving(Show)]
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
}

bitflags! {
    flags KeyModifiers: u8 {
        const SHIFT = 0x01,
        const CONTROL = 0x02,
        const ALT = 0x04,
        const SUPER = 0x08,
    }
}

/// Messages from the compositor and script to the constellation.
pub enum Msg {
    ExitMsg,
    FailureMsg(Failure),
    InitLoadUrlMsg(Url),
    LoadCompleteMsg,
    FrameRectMsg(PipelineId, SubpageId, Rect<f32>),
    LoadUrlMsg(PipelineId, LoadData),
    ScriptLoadedURLInIFrameMsg(Url, PipelineId, SubpageId, IFrameSandboxState),
    NavigateMsg(NavigationDirection),
    PainterReadyMsg(PipelineId),
    ResizedWindowMsg(WindowSizeData),
    KeyEvent(Key, KeyState, KeyModifiers),
    /// Requests that the constellation inform the compositor of the title of the pipeline
    /// immediately.
    GetPipelineTitleMsg(PipelineId),
}

/// Similar to net::resource_task::LoadData
/// can be passed to LoadUrlMsg to load a page with GET/POST
/// parameters or headers
#[deriving(Clone)]
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
            method: Get,
            headers: Headers::new(),
            data: None,
        }
    }
}

/// Represents the two different ways to which a page can be navigated
#[deriving(Clone, PartialEq, Hash, Show)]
pub enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

#[deriving(Clone, PartialEq, Hash, Show)]
pub enum NavigationDirection {
    Forward,
    Back,
}

#[deriving(Clone, PartialEq, Eq, Hash, Show)]
pub struct PipelineId(pub uint);

#[deriving(Clone, PartialEq, Eq, Hash, Show)]
pub struct SubpageId(pub uint);
