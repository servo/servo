/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo, the mighty web browser engine from the future.
//!
//! This crate wires all of Servo's components together as type [`Servo`], along with
//! a Webview implementation, [`WebView`], to create a working web engine.
//!
//! To embed Servo in your application, you need to:
//! 1) Create an instance of a type that implements the [`EventLoopWaker`] trait.
//!    This trait allows Servo to integrate with your application's event loop. Your
//!    [`EventLoopWaker::wake`] implementation needs to ensure that
//!    [`Servo::spin_event_loop`] is eventually called to let Servo process user input,
//!    network events etc.
//! 2) Create a [`Servo`] instance using the [`ServoBuilder`] type. You can optionally
//!    register a [`ServoDelegate`] implementation to subscribe to notifications about
//!    events and customize certain behaviours.
//! 3) Create an instance of a type that implements the [`RenderingContext`] trait.
//!    Note: You can either use one of the existing types in this crate (such as
//!    [`WindowRenderingContext`], [`OffscreenRenderingContext`] or [`SoftwareRenderingContext`])
//!    or provide a custom implementation.
//! 4) For each [`WebView`] in your application, create a [`WebViewBuilder`] by passing
//!    it the [`RenderingContext`] and [`Servo`] instance, and then configure the
//!    builder and use it to create a [`WebView`] instance. The builder must be provided
//!    with a [`WebViewDelegate`] implementation which, at a minimum, should handle
//!    [`WebViewDelegate::notify_new_frame_ready`] by calling [`WebView::paint`]
//!    and presenting the rendered page using [`RenderingContext::present`]. Refer to the
//!    documentation of the [`WebView`] type to learn more about the relation
//!    between a [`WebView`] and its [`RenderingContext`].
//! 5) Run the application's event loop. Your application's event handlers need to
//!    forward input events for a particular [`WebView`] using one of the `notify_*` methods
//!    on that [`WebView`] instance. For example, to forward a mouse event to a [`WebView`],
//!    call the [`WebView::notify_input_event`]. You can also invoke methods on a [`WebView`]
//!    to request certain actions. For instance, the [`WebView::load`] method requests that
//!    Servo navigate to a new page. In both cases, the calls to the [`WebView`] methods
//!    must be followed by calls to [`Servo::spin_event_loop`] to allow Servo to process
//!    those requests.
//!
//! For a minimal working example, refer to the [`winit_minimal`] code.
//!
//! [`winit_minimal`]: https://github.com/servo/servo/blob/main/components/servo/examples/winit_minimal.rs

mod clipboard_delegate;
#[cfg(feature = "gamepad")]
mod gamepad_delegate;
#[cfg(feature = "media-gstreamer")]
mod gstreamer_plugins;
mod javascript_evaluator;
mod network_manager;
mod proxies;
mod responders;
mod servo;
mod servo_delegate;
mod site_data_manager;
mod user_content_manager;
mod webview;
mod webview_delegate;

// These are Servo's public exports. Everything (apart from a couple exceptions below)
// should be exported at the root. See <https://github.com/servo/servo/issues/18475>.
pub use accesskit;
pub use embedder_traits::user_contents::UserScript;
pub use embedder_traits::{submit_resource_reader, *};
pub use image::RgbaImage;
pub use keyboard_types::{
    Code, CompositionEvent, CompositionState, Key, KeyState, Location, Modifiers, NamedKey,
};
pub use media::{
    GlApi as MediaGlApi, GlContext as MediaGlContext, NativeDisplay as MediaNativeDisplay,
};
pub use net::image_cache::should_panic_hook_suppress_termination;
pub use net_traits::CookieSource;
// This API should probably not be exposed in this way. Instead there should be a fully
// fleshed out public domains API if we want to expose it.
pub use net_traits::pub_domains::is_reg_domain;
pub use paint::WebRenderDebugOption;
pub use paint_api::rendering_context::{
    OffscreenRenderingContext, RenderingContext, SoftwareRenderingContext, WindowRenderingContext,
};
// Expose our profile traits for servoshell, so we can instrument code there, but don't
// add it as an official API.
#[doc(hidden)]
pub use profile_traits;
// This should be replaced with an API on ServoBuilder.
// See <https://github.com/servo/servo/issues/40950>.
pub use resources;
pub use servo_base::generic_channel::GenericSender;
pub use servo_base::id::WebViewId;
pub use servo_config::opts::{DiagnosticsLogging, DiagnosticsLoggingOption, Opts, OutputOptions};
pub use servo_config::prefs::{PrefValue, Preferences, UserAgentPlatform};
pub use servo_config::{opts, pref, prefs};
pub use servo_geometry::{
    DeviceIndependentIntRect, DeviceIndependentPixel, convert_rect_to_css_pixel,
};
#[doc(hidden)]
pub use servo_tracing;
pub use servo_url::ServoUrl;
pub use style::Zero;
pub use style_traits::CSSPixel;
pub use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel, DevicePoint, DeviceVector2D,
};

pub use crate::clipboard_delegate::{ClipboardDelegate, StringRequest};
#[cfg(feature = "gamepad")]
pub use crate::gamepad_delegate::{
    GamepadDelegate, GamepadHapticEffectRequest, GamepadHapticEffectRequestType,
};
pub use crate::network_manager::{CacheEntry, NetworkManager};
pub use crate::servo::{Servo, ServoBuilder, run_content_process};
pub use crate::servo_delegate::{ServoDelegate, ServoError};
pub use crate::site_data_manager::{SiteData, SiteDataManager, StorageType};
pub use crate::user_content_manager::UserContentManager;
pub use crate::webview::{WebView, WebViewBuilder};
pub use crate::webview_delegate::{
    AlertDialog, AllowOrDenyRequest, AuthenticationRequest, BluetoothDeviceSelectionRequest,
    ColorPicker, ConfirmDialog, ContextMenu, CreateNewWebViewRequest, EmbedderControl, FilePicker,
    InputMethodControl, NavigationRequest, PermissionRequest, PromptDialog, SelectElement,
    SimpleDialog, WebResourceLoad, WebViewDelegate,
};

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
    pub use net_traits::filemanager_thread::RelativePos;
    pub use net_traits::http_status::HttpStatus;
    pub use net_traits::request::Request;
    pub use net_traits::response::{Response, ResponseBody};
    pub use net_traits::{NetworkError, ResourceFetchTiming};

    pub use crate::webview_delegate::ProtocolHandlerRegistration;
}

// We need to reference this crate, in order for the linker not to remove it.
#[cfg(all(feature = "baked-in-resources", not(target_env = "ohos")))]
use servo_default_resources as _;
