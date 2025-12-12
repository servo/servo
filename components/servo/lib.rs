/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo, the mighty web browser engine from the future.
//!
//! This is a very simple library that wires all of Servo's components together as
//! type `Servo`, along with a Webview implementation, `WebView` to create a working
//! web browser.
//!
//! The `Servo` type is responsible for configuring a `Constellation`, which does the
//! heavy lifting of coordinating all of Servo's internal subsystems, including the
//! `ScriptThread` and the `LayoutThread`, as well maintains the navigation context.

mod clipboard_delegate;
mod cookies;
mod javascript_evaluator;
mod proxies;
mod responders;
mod servo;
mod servo_delegate;
mod webview;
mod webview_delegate;

// These are Servo's public exports. Everything (apart from a couple exceptions below)
// should be exported at the root. See <https://github.com/servo/servo/issues/18475>.
pub use base::generic_channel::GenericSender;
pub use base::id::WebViewId;
pub use compositing::WebRenderDebugOption;
pub use compositing_traits::rendering_context::{
    OffscreenRenderingContext, RenderingContext, SoftwareRenderingContext, WindowRenderingContext,
};
pub use embedder_traits::user_content_manager::{UserContentManager, UserScript};
pub use embedder_traits::*;
pub use image::RgbaImage;
pub use ipc_channel::ipc::IpcSender;
pub use keyboard_types::{
    Code, CompositionEvent, CompositionState, Key, KeyState, Location, Modifiers, NamedKey,
};
pub use media::{
    GlApi as MediaGlApi, GlContext as MediaGlContext, NativeDisplay as MediaNativeDisplay,
};
// This API should probably not be exposed in this way. Instead there should be a fully
// fleshed out public domains API if we want to expose it.
pub use net_traits::pub_domains::is_reg_domain;
// This should be replaced with an API on ServoBuilder.
// See <https://github.com/servo/servo/issues/40950>.
pub use resources;
pub use servo_config::opts::{DiagnosticsLogging, Opts, OutputOptions};
pub use servo_config::prefs::{PrefValue, Preferences, UserAgentPlatform};
pub use servo_config::{opts, pref, prefs};
pub use servo_geometry::{
    DeviceIndependentIntRect, DeviceIndependentPixel, convert_rect_to_css_pixel,
};
pub use servo_url::ServoUrl;
pub use style::Zero;
pub use style_traits::CSSPixel;
pub use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel, DevicePoint, DeviceVector2D,
};

pub use crate::servo::{Servo, ServoBuilder, run_content_process};
pub use crate::servo_delegate::{ServoDelegate, ServoError};
pub use crate::webview::{WebView, WebViewBuilder};
pub use crate::webview_delegate::{
    AlertDialog, AllowOrDenyRequest, AuthenticationRequest, ColorPicker, ConfirmDialog,
    ContextMenu, EmbedderControl, FilePicker, InputMethodControl, NavigationRequest,
    PermissionRequest, PromptDialog, SelectElement, SimpleDialog, WebResourceLoad, WebViewDelegate,
};

// Since WebXR is guarded by conditional compilation it is exported via submodule.
#[cfg(feature = "webxr")]
pub mod webxr {
    #[cfg(not(any(target_os = "android", target_env = "ohos")))]
    pub use webxr::glwindow::{GlWindow, GlWindowDiscovery, GlWindowMode, GlWindowRenderTarget};
    #[cfg(not(any(target_os = "android", target_env = "ohos")))]
    pub use webxr::headless::HeadlessMockDiscovery;
    #[cfg(target_os = "windows")]
    pub use webxr::openxr::{AppInfo as OpenXrAppInfo, OpenXrDiscovery};
    pub use webxr::{Discovery, MainThreadRegistry, WebXrRegistry};
}

// TODO: The protocol handler interface needs to be cleaned and simplified.
pub mod protocol_handler {
    pub use net::fetch::methods::{DoneChannel, FetchContext};
    pub use net::filemanager_thread::FILE_CHUNK_SIZE;
    pub use net::protocols::{ProtocolHandler, ProtocolRegistry};
    pub use net_traits::ResourceFetchTiming;
    pub use net_traits::filemanager_thread::RelativePos;
    pub use net_traits::http_status::HttpStatus;
    pub use net_traits::request::Request;
    pub use net_traits::response::{Response, ResponseBody};

    pub use crate::webview_delegate::ProtocolHandlerRegistration;
}
