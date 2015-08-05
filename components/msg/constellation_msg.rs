/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps
//! reduce coupling between these two components.

use compositor_msg::Epoch;

use canvas_traits::CanvasMsg;
use euclid::rect::Rect;
use euclid::size::{Size2D, TypedSize2D};
use euclid::scale_factor::ScaleFactor;
use hyper::header::Headers;
use hyper::method::Method;
use ipc_channel::ipc::IpcSender;
use layers::geometry::DevicePixel;
use offscreen_gl_context::GLContextAttributes;
use png::Image;
use util::cursor::Cursor;
use util::geometry::{PagePx, ViewportPx};
use util::mem::HeapSizeOf;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use style::viewport::ViewportConstraints;
use url::Url;
use webdriver_msg::{WebDriverScriptCommand, LoadStatus};

#[derive(Clone)]
pub struct ConstellationChan(pub Sender<Msg>);

impl ConstellationChan {
    pub fn new() -> (Receiver<Msg>, ConstellationChan) {
        let (chan, port) = channel();
        (port, ConstellationChan(chan))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum IFrameSandboxState {
    IFrameSandboxed,
    IFrameUnsandboxed
}

// We pass this info to various tasks, so it lives in a separate, cloneable struct.
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Failure {
    pub pipeline_id: PipelineId,
    pub parent_info: Option<(PipelineId, SubpageId)>,
}

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

#[derive(PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
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

/// Specifies the type of focus event that is sent to a pipeline
#[derive(Copy, Clone, PartialEq)]
pub enum FocusType {
    Element,    // The first focus message - focus the element itself
    Parent,     // Focusing a parent element (an iframe)
}

/// Messages from the compositor and script to the constellation.
#[derive(Deserialize, Serialize)]
pub enum Msg {
    Exit,
    Failure(Failure),
    InitLoadUrl(Url),
    LoadComplete(PipelineId),
    /// Dispatched after the DOM load event has fired on a document
    DOMLoad(PipelineId),
    FrameRect(PipelineId, SubpageId, Rect<f32>),
    LoadUrl(PipelineId, LoadData),
    ScriptLoadedURLInIFrame(Url, PipelineId, SubpageId, Option<SubpageId>, IFrameSandboxState),
    Navigate(Option<(PipelineId, SubpageId)>, NavigationDirection),
    PainterReady(PipelineId),
    ResizedWindow(WindowSizeData),
    KeyEvent(Key, KeyState, KeyModifiers),
    /// Requests that the constellation inform the compositor of the title of the pipeline
    /// immediately.
    GetPipelineTitle(PipelineId),
    /// Requests that the constellation inform the compositor of the a cursor change.
    SetCursor(Cursor),
    /// Dispatch a mozbrowser event to a given iframe. Only available in experimental mode.
    MozBrowserEvent(PipelineId, SubpageId, MozBrowserEvent),
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId),
    /// Request that the constellation send the current pipeline id for the provided frame
    /// id, or for the root frame if this is None, over a provided channel
    GetPipeline(Option<FrameId>, IpcSender<Option<PipelineId>>),
    /// Request that the constellation send the FrameId corresponding to the document
    /// with the provided parent pipeline id and subpage id
    GetFrame(PipelineId, SubpageId, IpcSender<Option<FrameId>>),
    /// Notifies the constellation that this frame has received focus.
    Focus(PipelineId),
    /// Requests that the constellation retrieve the current contents of the clipboard
    GetClipboardContents(IpcSender<String>),
    /// Requests that the constellation set the contents of the clipboard
    SetClipboardContents(String),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Notifies the constellation that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    /// Notification that this iframe should be removed.
    RemoveIFrame(PipelineId, SubpageId),
    /// Favicon detected
    NewFavicon(Url),
    /// <head> tag finished parsing
    HeadParsed,
    /// Requests that a new 2D canvas thread be created. (This is done in the constellation because
    /// 2D canvases may use the GPU and we don't want to give untrusted content access to the GPU.)
    CreateCanvasPaintTask(Size2D<i32>, IpcSender<(IpcSender<CanvasMsg>, usize)>),
    /// Requests that a new WebGL thread be created. (This is done in the constellation because
    /// WebGL uses the GPU and we don't want to give untrusted content access to the GPU.)
    CreateWebGLPaintTask(Size2D<i32>,
                         GLContextAttributes,
                         IpcSender<Result<(IpcSender<CanvasMsg>, usize), String>>),
    /// Status message to be displayed in the chrome, eg. a link URL on mouseover.
    NodeStatus(Option<String>),
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum AnimationState {
    AnimationsPresent,
    AnimationCallbacksPresent,
    NoAnimationsPresent,
    NoAnimationCallbacksPresent,
}

// https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API#Events
#[derive(Deserialize, Serialize)]
pub enum MozBrowserEvent {
    /// Sent when the scroll position within a browser <iframe> changes.
    AsyncScroll,
    /// Sent when window.close() is called within a browser <iframe>.
    Close,
    /// Sent when a browser <iframe> tries to open a context menu. This allows
    /// handling <menuitem> element available within the browser <iframe>'s content.
    ContextMenu,
    /// Sent when an error occurred while trying to load content within a browser <iframe>.
    Error,
    /// Sent when the favicon of a browser <iframe> changes.
    IconChange,
    /// Sent when the browser <iframe> has finished loading all its assets.
    LoadEnd,
    /// Sent when the browser <iframe> starts to load a new page.
    LoadStart,
    /// Sent when a browser <iframe>'s location changes.
    LocationChange(String),
    /// Sent when window.open() is called within a browser <iframe>.
    OpenWindow,
    /// Sent when the SSL state changes within a browser <iframe>.
    SecurityChange,
    /// Sent when alert(), confirm(), or prompt() is called within a browser <iframe>.
    ShowModalPrompt,
    /// Sent when the document.title changes within a browser <iframe>.
    TitleChange(String),
    /// Sent when an HTTP authentification is requested.
    UsernameAndPasswordRequired,
    /// Sent when a link to a search engine is found.
    OpenSearch,
}

impl MozBrowserEvent {
    pub fn name(&self) -> &'static str {
        match *self {
            MozBrowserEvent::AsyncScroll => "mozbrowserasyncscroll",
            MozBrowserEvent::Close => "mozbrowserclose",
            MozBrowserEvent::ContextMenu => "mozbrowsercontextmenu",
            MozBrowserEvent::Error => "mozbrowsererror",
            MozBrowserEvent::IconChange => "mozbrowsericonchange",
            MozBrowserEvent::LoadEnd => "mozbrowserloadend",
            MozBrowserEvent::LoadStart => "mozbrowserloadstart",
            MozBrowserEvent::LocationChange(_) => "mozbrowserlocationchange",
            MozBrowserEvent::OpenWindow => "mozbrowseropenwindow",
            MozBrowserEvent::SecurityChange => "mozbrowsersecuritychange",
            MozBrowserEvent::ShowModalPrompt => "mozbrowsershowmodalprompt",
            MozBrowserEvent::TitleChange(_) => "mozbrowsertitlechange",
            MozBrowserEvent::UsernameAndPasswordRequired => "mozbrowserusernameandpasswordrequired",
            MozBrowserEvent::OpenSearch => "mozbrowseropensearch"
        }
    }
    pub fn detail(&self) -> Option<String> {
        match *self {
            MozBrowserEvent::AsyncScroll | MozBrowserEvent::Close | MozBrowserEvent::ContextMenu |
            MozBrowserEvent::Error | MozBrowserEvent::IconChange | MozBrowserEvent::LoadEnd |
            MozBrowserEvent::LoadStart | MozBrowserEvent::OpenWindow | MozBrowserEvent::SecurityChange |
            MozBrowserEvent::ShowModalPrompt | MozBrowserEvent::UsernameAndPasswordRequired |
            MozBrowserEvent::OpenSearch => None,
            MozBrowserEvent::LocationChange(ref new_location) => Some(new_location.clone()),
            MozBrowserEvent::TitleChange(ref new_title) => Some(new_title.clone()),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    LoadUrl(PipelineId, LoadData, IpcSender<LoadStatus>),
    Refresh(PipelineId, IpcSender<LoadStatus>),
    ScriptCommand(PipelineId, WebDriverScriptCommand),
    TakeScreenshot(PipelineId, IpcSender<Option<Image>>)
}

/// Similar to net::resource_task::LoadData
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

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct WorkerId(pub u32);

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct PipelineId(pub u32);

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct SubpageId(pub u32);

// The type of pipeline exit. During complete shutdowns, pipelines do not have to
// release resources automatically released on process termination.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum PipelineExitType {
    PipelineOnly,
    Complete,
}
