/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Mouse press/release/move and wheel (axis) input for the OpenHarmony embedder.
//!
//! The whole feature lives in this file; [`super`] only declares the module, extends
//! [`super::ServoAction`] with the two actions that are needed to hand the event to the servo
//! main thread, and registers the callbacks. Nothing that already existed in `mod.rs` is
//! rewritten, reordered or reformatted.

use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::sync::Mutex;

use log::{debug, error, warn};
use servo::{DevicePoint, InputEvent, MouseButton, MouseLeftViewportEvent, WheelEvent};
use xcomponent_sys::{
    OH_NativeXComponent, OH_NativeXComponent_GetMouseEvent, OH_NativeXComponent_MouseEvent,
    OH_NativeXComponent_MouseEvent_Callback, OH_NativeXComponent_RegisterMouseEventCallback,
};

use super::{App, ServoAction, call};
use crate::wheel::{AxisEventSource, wheel_delta_from_axis};

/// Opaque handle to an ArkUI UI input event (`ArkUI_UIInputEvent`).
///
/// We only ever pass this pointer back to the `OH_ArkUI_*` accessor functions, so the
/// layout does not matter here. (`xcomponent-sys` does not declare it, hence the local
/// declaration.)
#[repr(C)]
pub struct ArkUI_UIInputEvent {
    _private: [u8; 0],
}

/// `ARKUI_UIINPUTEVENT_TYPE_AXIS` from `arkui/ui_input_event.h`.
const ARKUI_UIINPUTEVENT_TYPE_AXIS: i32 = 2;

/// `UI_INPUT_EVENT_TOOL_TYPE_*` from `arkui/ui_input_event.h`.
///
/// The tool type tells the two kinds of scroll input apart, which matters because the
/// platform reports them in different units and reference frames.
const UI_INPUT_EVENT_TOOL_TYPE_FINGER: i32 = 1;
const UI_INPUT_EVENT_TOOL_TYPE_TOUCHPAD: i32 = 4;

unsafe extern "C" {
    fn OH_ArkUI_AxisEvent_GetVerticalAxisValue(event: *const ArkUI_UIInputEvent) -> f64;
    fn OH_ArkUI_AxisEvent_GetHorizontalAxisValue(event: *const ArkUI_UIInputEvent) -> f64;
    fn OH_ArkUI_UIInputEvent_GetToolType(event: *const ArkUI_UIInputEvent) -> i32;
    fn OH_NativeXComponent_RegisterUIInputEventCallback(
        component: *mut OH_NativeXComponent,
        callback: Option<
            unsafe extern "C" fn(
                component: *mut OH_NativeXComponent,
                event: *mut ArkUI_UIInputEvent,
                event_type: i32,
            ),
        >,
        event_type: i32,
    ) -> i32;
}

/// Last known pointer position (vp), used as the anchor point of wheel events.
static LAST_POINTER: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

/// Maps `OH_NativeXComponent_MouseEventButton` onto servo's [`MouseButton`].
fn mouse_button(button: u32) -> MouseButton {
    match button {
        // OH_NATIVEXCOMPONENT_LEFT_BUTTON
        1 => MouseButton::Primary,
        // OH_NATIVEXCOMPONENT_RIGHT_BUTTON
        2 => MouseButton::Secondary,
        // OH_NATIVEXCOMPONENT_MIDDLE_BUTTON
        4 => MouseButton::Auxiliary,
        _ => MouseButton::None,
    }
}

/// Pushes the page's CSS cursor to ArkTS, but only when it actually changed.
fn dispatch_cursor(app: &App) {
    if let Some(webview) = app.active_or_newest_webview() {
        super::host_callbacks::notify_cursor(webview.cursor());
    }
}

/// Dispatches an `OH_NativeXComponent_MouseEvent` to servo.
pub(super) fn dispatch_mouse_event(app: &App, action: u32, button: u32, x: f32, y: f32) {
    match action {
        // OH_NATIVEXCOMPONENT_MOUSE_PRESS
        1 => app.mouse_down(x, y, mouse_button(button)),
        // OH_NATIVEXCOMPONENT_MOUSE_RELEASE
        2 => app.mouse_up(x, y, mouse_button(button)),
        // OH_NATIVEXCOMPONENT_MOUSE_MOVE - also drives CSS `:hover` and the cursor.
        3 => {
            app.mouse_move(x, y);
            dispatch_cursor(app);
        },
        _ => {},
    }
}

/// Dispatches a wheel / touchpad scroll to servo, anchored at the last pointer position.
///
/// The platform reports the scroll as an angle in degrees for a mouse wheel and as a
/// distance in pixels for a touchpad, which [`wheel_delta_from_axis`] converts into the
/// device-pixel delta that [`servo::InputEvent::Wheel`] expects.
pub(super) fn dispatch_wheel_event(
    app: &App,
    horizontal: f32,
    vertical: f32,
    source: AxisEventSource,
) {
    let delta = wheel_delta_from_axis(horizontal, vertical, source);
    let (x, y) = *LAST_POINTER.lock().unwrap();
    let point = DevicePoint::new(x, y).into();
    dispatch_input_event(app, InputEvent::Wheel(WheelEvent::new(delta, point)));
}

/// Dispatches a hover change of the pointer to servo.
///
/// Servo only needs to know when the pointer leaves the viewport, hover updates inside the
/// viewport are covered by the mouse move events.
pub(super) fn dispatch_hover_event(app: &App, is_hover: bool) {
    if !is_hover {
        dispatch_input_event(
            app,
            InputEvent::MouseLeftViewport(MouseLeftViewportEvent::default()),
        );
    }
}

/// Sends an input event to the active webview, and lets the embedder process it right away.
///
/// This is the same sequence that the input entry points of `App` perform (`App::mouse_move()`,
/// `App::touch_down()`, ...), so the mouse and wheel input of this embedder reaches servo through
/// the same `WebView::notify_input_event()` path as the input of the desktop port.
fn dispatch_input_event(app: &App, event: InputEvent) {
    if let Some(webview) = app.active_or_newest_webview() {
        webview.notify_input_event(event);
        app.spin_event_loop();
    }
}

extern "C" fn on_dispatch_mouse_event_cb(component: *mut OH_NativeXComponent, window: *mut c_void) {
    let mut mouse_event: MaybeUninit<OH_NativeXComponent_MouseEvent> = MaybeUninit::uninit();
    let res =
        unsafe { OH_NativeXComponent_GetMouseEvent(component, window, mouse_event.as_mut_ptr()) };
    if res != 0 {
        error!("OH_NativeXComponent_GetMouseEvent failed with {res}");
        return;
    }
    let mouse_event = unsafe { mouse_event.assume_init() };
    *LAST_POINTER.lock().unwrap() = (mouse_event.x, mouse_event.y);
    if let Err(e) = call(ServoAction::MouseEvent {
        action: mouse_event.action.0,
        button: mouse_event.button.0,
        x: mouse_event.x,
        y: mouse_event.y,
    }) {
        error!("Failed to dispatch mouse event: {e:?}");
    }
}

/// Called when the mouse pointer enters or leaves the `XComponent`.
extern "C" fn on_hover_event_cb(_component: *mut OH_NativeXComponent, is_hover: bool) {
    if let Err(e) = call(ServoAction::HoverChanged { is_hover }) {
        error!("Failed to dispatch hover event: {e:?}");
    }
}

/// Wheel and touchpad scrolling arrive through the generic ArkUI axis-event callback:
/// `OH_NativeXComponent_MouseEvent` has no wheel action of its own.
unsafe extern "C" fn on_axis_event_cb(
    _component: *mut OH_NativeXComponent,
    event: *mut ArkUI_UIInputEvent,
    _event_type: i32,
) {
    if event.is_null() {
        return;
    }
    let vertical = unsafe { OH_ArkUI_AxisEvent_GetVerticalAxisValue(event) } as f32;
    let horizontal = unsafe { OH_ArkUI_AxisEvent_GetHorizontalAxisValue(event) } as f32;
    if vertical == 0.0 && horizontal == 0.0 {
        return;
    }
    if let Err(e) = call(ServoAction::WheelEvent {
        horizontal,
        vertical,
        source: axis_event_source(event),
    }) {
        error!("Failed to dispatch wheel event: {e:?}");
    }
}

/// Classifies a scroll axis event into the source whose unit and frame it is reported in.
///
/// `arkui/ui_input_event.h` reports mouse wheel scrolling in degrees in the frame of the
/// viewport, and touchpad scrolling in pixels in the frame of the finger, so the
/// conversion has to know which device produced the event. Anything that is not reported
/// as a touchpad keeps the mouse wheel interpretation, which is what this embedder used
/// before the two sources were told apart.
fn axis_event_source(event: *const ArkUI_UIInputEvent) -> AxisEventSource {
    match unsafe { OH_ArkUI_UIInputEvent_GetToolType(event) } {
        // A touchpad may report its gestures as finger movement.
        UI_INPUT_EVENT_TOOL_TYPE_TOUCHPAD | UI_INPUT_EVENT_TOOL_TYPE_FINGER => {
            AxisEventSource::Touchpad
        },
        _ => AxisEventSource::MouseWheel,
    }
}

/// Registers the mouse and the wheel (axis) callbacks on the `XComponent`.
pub(crate) fn register_callbacks(nativeXComponent: *mut OH_NativeXComponent) {
    let mouse_cbs = Box::new(OH_NativeXComponent_MouseEvent_Callback {
        DispatchMouseEvent: Some(on_dispatch_mouse_event_cb),
        DispatchHoverEvent: Some(on_hover_event_cb),
    });
    let res = unsafe {
        OH_NativeXComponent_RegisterMouseEventCallback(
            nativeXComponent,
            Box::leak(mouse_cbs) as *mut _,
        )
    };
    if res != 0 {
        error!("Failed to register mouse event callbacks with {res}");
    } else {
        debug!("Registered mouse event callbacks successfully");
    }

    let res = unsafe {
        OH_NativeXComponent_RegisterUIInputEventCallback(
            nativeXComponent,
            Some(on_axis_event_cb),
            ARKUI_UIINPUTEVENT_TYPE_AXIS,
        )
    };
    if res != 0 {
        // Not fatal: without this callback the app still works, it just cannot scroll with a wheel.
        warn!("Failed to register wheel (axis) event callback with {res}");
    } else {
        debug!("Registered wheel (axis) event callback successfully");
    }
}
