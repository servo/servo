/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Experimental in-process CPython bindings for the Severin local runtime.

mod bridge_trace;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString, c_char, c_int, c_long, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use servo::{
    EventLoopWaker, JSValue, JavaScriptEvaluationError, LoadStatus, Preferences, RenderingContext,
    Servo, ServoBuilder, UserContentManager, UserScript, WebView, WebViewBuilder,
    WindowRenderingContext,
};
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const DEFAULT_PACKAGE_ID: &str = "com.example.app";
const PY_TPFLAGS_DEFAULT: u64 = 0;
const PY_TPFLAGS_BASETYPE: u64 = 1 << 10;
const PY_MOD_EXEC: c_int = 2;
const PY_TP_NEW: c_int = 65;
const PY_TP_INIT: c_int = 60;
const PY_TP_DEALLOC: c_int = 52;
const PY_TP_METHODS: c_int = 64;
const METH_NOARGS: c_int = 0x0004;
const METH_O: c_int = 0x0008;
const METH_VARARGS: c_int = 0x0001;

#[repr(C)]
struct PyObject {
    _private: [u8; 0],
}
#[repr(C)]
struct PyTypeObject {
    _private: [u8; 0],
}
#[repr(C)]
struct PyModuleDef {
    base: PyModuleDef_Base,
    name: *const c_char,
    doc: *const c_char,
    size: isize,
    methods: *mut PyMethodDef,
    slots: *mut PyModuleDef_Slot,
    traverse: *mut c_void,
    clear: *mut c_void,
    free: *mut c_void,
}
#[repr(C)]
struct PyModuleDef_Base {
    ob_base: [usize; 2],
    init: *mut c_void,
    index: isize,
    copy: *mut PyObject,
}
#[repr(C)]
struct PyModuleDef_Slot {
    slot: c_int,
    value: *mut c_void,
}
#[repr(C)]
struct PyType_Spec {
    name: *const c_char,
    basicsize: c_int,
    itemsize: c_int,
    flags: u64,
    slots: *mut PyType_Slot,
}
#[repr(C)]
struct PyType_Slot {
    slot: c_int,
    pfunc: *mut c_void,
}
#[repr(C)]
struct PyMethodDef {
    ml_name: *const c_char,
    ml_meth: *mut c_void,
    ml_flags: c_int,
    ml_doc: *const c_char,
}
#[repr(C)]
struct PyAppObject {
    ob_base: [usize; 2],
    app: *mut EmbeddedServoApp,
    bridge: *mut PyObject,
    closed: bool,
}

unsafe extern "C" {
    static mut PyExc_RuntimeError: *mut PyObject;
    static mut PyExc_ValueError: *mut PyObject;
    fn PyModuleDef_Init(def: *mut PyModuleDef) -> *mut PyObject;
    fn PyModule_AddObject(
        module: *mut PyObject,
        name: *const c_char,
        value: *mut PyObject,
    ) -> c_int;
    fn PyType_FromSpec(spec: *mut PyType_Spec) -> *mut PyObject;
    fn PyType_GenericNew(
        subtype: *mut PyTypeObject,
        args: *mut PyObject,
        kwds: *mut PyObject,
    ) -> *mut PyObject;
    fn PyArg_ParseTuple(args: *mut PyObject, format: *const c_char, ...) -> c_int;
    fn PyArg_ParseTupleAndKeywords(
        args: *mut PyObject,
        kwds: *mut PyObject,
        format: *const c_char,
        kwlist: *mut *mut c_char,
        ...
    ) -> c_int;
    fn PyErr_SetString(exception: *mut PyObject, string: *const c_char);
    fn Py_IncRef(object: *mut PyObject);
    fn Py_DecRef(object: *mut PyObject);
    fn PyLong_AsLong(object: *mut PyObject) -> c_long;
    fn PyLong_AsUnsignedLongLong(object: *mut PyObject) -> u64;
    fn PyLong_FromUnsignedLongLong(value: u64) -> *mut PyObject;
    fn PyTuple_New(size: isize) -> *mut PyObject;
    fn PyTuple_SetItem(tuple: *mut PyObject, position: isize, item: *mut PyObject) -> c_int;
    fn PyUnicode_FromStringAndSize(string: *const c_char, size: isize) -> *mut PyObject;
    fn PyUnicode_AsUTF8(object: *mut PyObject) -> *const c_char;
    static mut _Py_NoneStruct: PyObject;
}

#[derive(Debug)]
struct SeverinWakerEvent;

#[derive(Clone)]
struct WinitEventLoopWaker {
    proxy: EventLoopProxy<SeverinWakerEvent>,
    wake_flag: Arc<AtomicBool>,
}

impl EventLoopWaker for WinitEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }
    fn wake(&self) {
        self.wake_flag.store(true, Ordering::Relaxed);
        let _ = self.proxy.send_event(SeverinWakerEvent);
    }
}

struct EmbeddedServoApp {
    presentation: WinitPresentation,
    servo: Servo,
    webview: WebView,
    _rendering_context: Rc<dyn RenderingContext>,
    _user_content_manager: Rc<UserContentManager>,
    wake_flag: Arc<AtomicBool>,
    bridge_transport: BridgeTransport,
    pending_evaluations: Vec<PendingEvaluation>,
    trace_bridge: bool,
    next_evaluation_id: u64,
}

struct WinitPresentation {
    event_loop: EventLoop<SeverinWakerEvent>,
    window: Window,
    rendering_context: Rc<WindowRenderingContext>,
    closed_by_window: bool,
}

struct WinitBootstrap {
    width: u32,
    height: u32,
    window: Option<Window>,
    rendering_context: Option<Rc<WindowRenderingContext>>,
    error: Option<String>,
}

impl ApplicationHandler<SeverinWakerEvent> for WinitBootstrap {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() || self.error.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Severin")
            .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                self.error = Some(format!("failed to create Severin window: {error}"));
                event_loop.exit();
                return;
            },
        };
        let display_handle = match event_loop.display_handle() {
            Ok(handle) => handle,
            Err(error) => {
                self.error = Some(format!("failed to get display handle: {error}"));
                event_loop.exit();
                return;
            },
        };
        let window_handle = match window.window_handle() {
            Ok(handle) => handle,
            Err(error) => {
                self.error = Some(format!("failed to get window handle: {error}"));
                event_loop.exit();
                return;
            },
        };
        let rendering_context =
            match WindowRenderingContext::new(display_handle, window_handle, window.inner_size()) {
                Ok(context) => Rc::new(context),
                Err(error) => {
                    self.error = Some(format!(
                        "failed to create window rendering context: {error:?}"
                    ));
                    event_loop.exit();
                    return;
                },
            };
        if let Err(error) = rendering_context.make_current() {
            self.error = Some(format!(
                "failed to make window rendering context current: {error:?}"
            ));
            event_loop.exit();
            return;
        }
        window.set_visible(true);
        self.rendering_context = Some(rendering_context);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
    }
}

struct WinitPump<'a> {
    servo: &'a Servo,
    webview: &'a WebView,
    window: &'a Window,
    rendering_context: &'a WindowRenderingContext,
    closed_by_window: &'a mut bool,
}

impl ApplicationHandler<SeverinWakerEvent> for WinitPump<'_> {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _event: SeverinWakerEvent,
    ) {
        self.servo.spin_event_loop();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                *self.closed_by_window = true;
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.webview.paint();
                self.rendering_context.present();
            },
            WindowEvent::Resized(size) => {
                self.webview.resize(size);
                self.window.request_redraw();
            },
            _ => {},
        }
        self.servo.spin_event_loop();
    }
}

#[derive(Default)]
struct BridgeTransport {
    next_receipt: u64,
    inbound: VecDeque<BridgeFrame>,
    pending: HashMap<u64, PendingReplyTarget>,
    active_document_id: Option<String>,
}

struct BridgeFrame {
    receipt: u64,
    json: String,
}

struct PendingReplyTarget {
    document_id: String,
    call_id: u64,
}

#[derive(Clone, Copy, Debug)]
enum PendingEvaluationKind {
    DrainOutbound,
    DeliverReply,
}

impl PendingEvaluationKind {
    fn label(self) -> &'static str {
        match self {
            Self::DrainOutbound => "drain",
            Self::DeliverReply => "reply",
        }
    }
}

struct PendingEvaluation {
    id: u64,
    kind: PendingEvaluationKind,
    result: Rc<RefCell<Option<Result<JSValue, JavaScriptEvaluationError>>>>,
}

fn trace_evaluation_callback(
    enabled: bool,
    evaluation_id: u64,
    kind: PendingEvaluationKind,
    result: &Result<JSValue, JavaScriptEvaluationError>,
) {
    match result {
        Ok(JSValue::String(serialized)) => bridge_trace::emit(
            enabled,
            format_args!(
                "eval:{evaluation_id} callback kind={} result=string bytes={}",
                kind.label(),
                serialized.len()
            ),
        ),
        Ok(value) => bridge_trace::emit(
            enabled,
            format_args!(
                "eval:{evaluation_id} callback kind={} result=unexpected value={value:?}",
                kind.label()
            ),
        ),
        Err(error) => bridge_trace::emit(
            enabled,
            format_args!(
                "eval:{evaluation_id} callback kind={} result=error error={error:?}",
                kind.label()
            ),
        ),
    }
}

impl BridgeTransport {
    fn enqueue_from_javascript(
        &mut self,
        document_id: String,
        call_id: u64,
        json: String,
    ) -> Result<u64, String> {
        validate_json_frame(&json)?;
        self.active_document_id = Some(document_id.clone());
        self.next_receipt = self.next_receipt.saturating_add(1);
        let receipt = self.next_receipt;
        self.pending.insert(
            receipt,
            PendingReplyTarget {
                document_id,
                call_id,
            },
        );
        self.inbound.push_back(BridgeFrame { receipt, json });
        Ok(receipt)
    }

    fn read_for_python(&mut self) -> Option<BridgeFrame> {
        self.inbound.pop_front()
    }

    fn prepare_reply(&mut self, receipt: u64, json: &str) -> Result<String, String> {
        validate_json_frame(json)?;
        let Some(target) = self.pending.get(&receipt) else {
            return Err("bridge reply target no longer exists".to_owned());
        };
        if self.active_document_id.as_deref() != Some(target.document_id.as_str()) {
            return Err("bridge reply target document is no longer active".to_owned());
        }
        let script = resolve_script(&target.document_id, target.call_id, json);
        self.pending.remove(&receipt);
        Ok(script)
    }

    fn finish_reply_delivery(&mut self, delivered: bool) -> Result<(), String> {
        if delivered {
            Ok(())
        } else {
            Err("bridge reply target was not found in the active document".to_owned())
        }
    }

    fn clear_for_navigation(&mut self) {
        self.inbound.clear();
        self.pending.clear();
        self.active_document_id = None;
    }

    fn clear_for_close(&mut self) {
        self.clear_for_navigation();
    }
}

fn validate_json_frame(json: &str) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(json)
        .map(|_| ())
        .map_err(|error| format!("invalid JSON bridge frame: {error}"))
}
const SEVERIN_BRIDGE_SHIM: &str = r#"
(() => {
  const documentId = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  let nextCallId = 1;
  const outbound = [];
  const pending = new Map();

  function rejectNotJson() {
    return Promise.reject(new TypeError("severin.send(value) requires a strict JSON value"));
  }

  function send(value) {
    let json;
    try {
      json = JSON.stringify(value);
    } catch (_) {
      return rejectNotJson();
    }
    if (typeof json !== "string") {
      return rejectNotJson();
    }

    const callId = nextCallId++;
    outbound.push({ callId, json });
    return new Promise((resolve, reject) => {
      pending.set(callId, { resolve, reject });
    });
  }

  Object.defineProperty(globalThis, "severin", {
    value: Object.freeze({ send }),
    configurable: false,
    enumerable: false,
    writable: false,
  });

  Object.defineProperty(globalThis, "__severinDrain", {
    value() {
      return { documentId, frames: outbound.splice(0, outbound.length) };
    },
    configurable: false,
    enumerable: false,
    writable: false,
  });

  Object.defineProperty(globalThis, "__severinResolve", {
    value(expectedDocumentId, callId, jsonSource) {
      if (expectedDocumentId !== documentId) {
        return false;
      }
      const target = pending.get(callId);
      if (!target) {
        return false;
      }
      let value;
      try {
        value = JSON.parse(jsonSource);
      } catch (error) {
        target.reject(error);
        pending.delete(callId);
        return false;
      }
      pending.delete(callId);
      target.resolve(value);
      return true;
    },
    configurable: false,
    enumerable: false,
    writable: false,
  });
})();
"#;

const DRAIN_SCRIPT: &str = r#"
(() => {
  if (typeof globalThis.__severinDrain !== "function") {
    return JSON.stringify({ documentId: null, frames: [] });
  }
  return JSON.stringify(globalThis.__severinDrain());
})()
"#;

fn resolve_script(document_id: &str, call_id: u64, json: &str) -> String {
    let document_id_literal = serde_json::to_string(document_id).expect("document id serializes");
    // The reply JSON is embedded as a JavaScript string literal and parsed by
    // the page shim; it is never concatenated into executable JavaScript as
    // application-controlled source.
    let json_literal = serde_json::to_string(json).expect("JSON source string serializes");
    format!(
        r#"(() => {{
  if (typeof globalThis.__severinResolve !== "function") {{
    return false;
  }}
  return globalThis.__severinResolve({document_id_literal}, {call_id}, {json_literal});
}})()"#
    )
}

impl EmbeddedServoApp {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let trace_bridge = bridge_trace::enabled();
        bridge_trace::emit(
            trace_bridge,
            format_args!("app:new width={width} height={height}"),
        );

        let mut event_loop = EventLoop::with_user_event()
            .build()
            .map_err(|error| format!("failed to create Severin event loop: {error}"))?;
        let mut bootstrap = WinitBootstrap {
            width,
            height,
            window: None,
            rendering_context: None,
            error: None,
        };
        while bootstrap.window.is_none() && bootstrap.error.is_none() {
            match event_loop.pump_app_events(Some(Duration::ZERO), &mut bootstrap) {
                PumpStatus::Continue => {},
                PumpStatus::Exit(_) => break,
            }
        }
        if let Some(error) = bootstrap.error {
            return Err(error);
        }
        let Some(window) = bootstrap.window.take() else {
            return Err("failed to create Severin window".to_owned());
        };
        let Some(rendering_context) = bootstrap.rendering_context.take() else {
            return Err("failed to create Severin window rendering context".to_owned());
        };
        let wake_flag = Arc::new(AtomicBool::new(false));
        let mut preferences = Preferences::default();
        preferences.network_http_proxy_uri = String::new();
        preferences.network_https_proxy_uri = String::new();
        let servo = ServoBuilder::default()
            .preferences(preferences)
            .event_loop_waker(Box::new(WinitEventLoopWaker {
                proxy: event_loop.create_proxy(),
                wake_flag: wake_flag.clone(),
            }))
            .build();
        let user_content_manager = Rc::new(UserContentManager::new(&servo));
        user_content_manager.add_script(Rc::new(UserScript::from(SEVERIN_BRIDGE_SHIM)));
        bridge_trace::emit(
            trace_bridge,
            format_args!("bridge-shim: queued in user-content manager"),
        );
        let webview = WebViewBuilder::new(&servo, rendering_context.clone())
            .user_content_manager(user_content_manager.clone())
            .build();
        bridge_trace::emit(trace_bridge, format_args!("app:new webview constructed"));
        Ok(Self {
            presentation: WinitPresentation {
                event_loop,
                window,
                rendering_context: rendering_context.clone(),
                closed_by_window: false,
            },
            servo,
            webview,
            _rendering_context: rendering_context,
            _user_content_manager: user_content_manager,
            wake_flag,
            bridge_transport: BridgeTransport::default(),
            pending_evaluations: Vec::new(),
            trace_bridge,
            next_evaluation_id: 0,
        })
    }

    fn trace(&self, message: std::fmt::Arguments<'_>) {
        bridge_trace::emit(self.trace_bridge, message);
    }

    fn next_evaluation_id(&mut self) -> u64 {
        self.next_evaluation_id = self.next_evaluation_id.saturating_add(1);
        self.next_evaluation_id
    }

    fn spin_once(&mut self) {
        self.wake_flag.store(false, Ordering::Relaxed);
        {
            let mut pump = WinitPump {
                servo: &self.servo,
                webview: &self.webview,
                window: &self.presentation.window,
                rendering_context: &self.presentation.rendering_context,
                closed_by_window: &mut self.presentation.closed_by_window,
            };
            let _ = self
                .presentation
                .event_loop
                .pump_app_events(Some(Duration::ZERO), &mut pump);
        }
        self.servo.spin_event_loop();
        if self.wake_flag.load(Ordering::Relaxed) {
            self.presentation.window.request_redraw();
        }
    }

    fn pump_once(&mut self) -> Result<(), String> {
        self.spin_once();
        self.collect_bridge_evaluations()?;
        self.schedule_outbound_drain();
        Ok(())
    }

    fn schedule_outbound_drain(&mut self) {
        if self
            .pending_evaluations
            .iter()
            .any(|evaluation| matches!(evaluation.kind, PendingEvaluationKind::DrainOutbound))
        {
            return;
        }

        let evaluation_id = self.next_evaluation_id();
        let kind = PendingEvaluationKind::DrainOutbound;
        self.trace(format_args!(
            "eval:{evaluation_id} queue kind={} load_complete={}",
            kind.label(),
            self.webview.load_status() == LoadStatus::Complete,
        ));
        let result = Rc::new(RefCell::new(None));
        let callback_result = result.clone();
        let trace_bridge = self.trace_bridge;
        self.webview
            .evaluate_javascript(DRAIN_SCRIPT, move |value| {
                trace_evaluation_callback(trace_bridge, evaluation_id, kind, &value);
                *callback_result.borrow_mut() = Some(value);
            });
        self.pending_evaluations.push(PendingEvaluation {
            id: evaluation_id,
            kind,
            result,
        });
    }

    fn schedule_reply_delivery(&mut self, script: String) {
        let evaluation_id = self.next_evaluation_id();
        let kind = PendingEvaluationKind::DeliverReply;
        self.trace(format_args!(
            "eval:{evaluation_id} queue kind={} script_bytes={}",
            kind.label(),
            script.len(),
        ));
        let result = Rc::new(RefCell::new(None));
        let callback_result = result.clone();
        let trace_bridge = self.trace_bridge;
        self.webview.evaluate_javascript(script, move |value| {
            trace_evaluation_callback(trace_bridge, evaluation_id, kind, &value);
            *callback_result.borrow_mut() = Some(value);
        });
        self.pending_evaluations.push(PendingEvaluation {
            id: evaluation_id,
            kind,
            result,
        });
    }

    fn collect_bridge_evaluations(&mut self) -> Result<(), String> {
        let mut index = 0;
        while index < self.pending_evaluations.len() {
            let result = { self.pending_evaluations[index].result.borrow_mut().take() };
            let Some(result) = result else {
                index += 1;
                continue;
            };
            let evaluation = self.pending_evaluations.remove(index);
            self.trace(format_args!(
                "eval:{} collect kind={}",
                evaluation.id,
                evaluation.kind.label()
            ));
            match evaluation.kind {
                PendingEvaluationKind::DrainOutbound => {
                    self.handle_drain_result(result)?;
                },
                PendingEvaluationKind::DeliverReply => {
                    let delivered = match result {
                        Ok(JSValue::Boolean(true)) => true,
                        Ok(value) => {
                            let message = format!(
                                "Severin bridge reply evaluation returned unexpected value: {value:?}"
                            );
                            self.trace(format_args!("reply: delivery failed error={message}"));
                            return Err(message);
                        },
                        Err(error) => {
                            let message =
                                format!("Severin bridge reply evaluation failed: {error:?}");
                            self.trace(format_args!("reply: delivery failed error={message}"));
                            return Err(message);
                        },
                    };
                    self.trace(format_args!("reply: delivery acknowledged"));
                    self.bridge_transport.finish_reply_delivery(delivered)?;
                },
            }
        }
        Ok(())
    }

    fn clear_bridge_for_navigation(&mut self) {
        self.trace(format_args!(
            "bridge: clear navigation pending_evaluations={} inbound={} replies={}",
            self.pending_evaluations.len(),
            self.bridge_transport.inbound.len(),
            self.bridge_transport.pending.len(),
        ));
        self.pending_evaluations.clear();
        self.bridge_transport.clear_for_navigation();
    }

    fn handle_drain_result(
        &mut self,
        result: Result<JSValue, JavaScriptEvaluationError>,
    ) -> Result<(), String> {
        let serialized = match result {
            Ok(JSValue::String(serialized)) => serialized,
            Ok(value) => {
                let message = format!(
                    "Severin bridge drain evaluation returned unexpected value: {value:?}"
                );
                self.trace(format_args!("drain: failed error={message}"));
                return Err(message);
            },
            Err(error) => {
                let message = format!("Severin bridge drain evaluation failed: {error:?}");
                self.trace(format_args!("drain: failed error={message}"));
                return Err(message);
            },
        };
        let drained: serde_json::Value = serde_json::from_str(&serialized)
            .map_err(|error| format!("invalid Severin bridge drain result: {error}"))?;
        let document_id = drained
            .get("documentId")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned();
        if document_id.is_empty() {
            self.trace(format_args!("drain: shim absent in evaluated document"));
            self.bridge_transport.clear_for_navigation();
            return Ok(());
        }
        if self.bridge_transport.active_document_id.as_deref() != Some(document_id.as_str()) {
            self.trace(format_args!("drain: active document changed id={document_id}"));
            self.bridge_transport.clear_for_navigation();
            self.bridge_transport.active_document_id = Some(document_id.clone());
        }
        let Some(frames) = drained.get("frames").and_then(|value| value.as_array()) else {
            let message = "Severin bridge drain result did not contain a frames array".to_owned();
            self.trace(format_args!("drain: failed error={message}"));
            return Err(message);
        };
        self.trace(format_args!(
            "drain: document={document_id} frames={}",
            frames.len()
        ));
        for frame in frames {
            let Some(call_id) = frame.get("callId").and_then(|value| value.as_u64()) else {
                self.trace(format_args!("drain: skipped malformed frame missing callId"));
                continue;
            };
            let Some(json) = frame.get("json").and_then(|value| value.as_str()) else {
                self.trace(format_args!("drain: skipped malformed frame call_id={call_id} missing json"));
                continue;
            };
            let receipt = self.bridge_transport.enqueue_from_javascript(
                document_id.clone(),
                call_id,
                json.to_owned(),
            )?;
            self.trace(format_args!(
                "frame: enqueue receipt={receipt} call_id={call_id} json_bytes={}",
                json.len()
            ));
        }
        Ok(())
    }
}

fn cstring_lossy(message: &str) -> CString {
    CString::new(message).unwrap_or_else(|_| CString::new("severin error contained NUL").unwrap())
}
unsafe fn set_error(exc: *mut PyObject, message: &str) {
    let c = cstring_lossy(message);
    unsafe { PyErr_SetString(exc, c.as_ptr()) };
}

unsafe extern "C" fn app_init(
    self_: *mut PyAppObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> c_int {
    let mut width_obj: *mut PyObject = ptr::null_mut();
    let mut height_obj: *mut PyObject = ptr::null_mut();
    let mut bridge: *mut PyObject = ptr::null_mut();
    if unsafe {
        PyArg_ParseTupleAndKeywords(
            args,
            kwds,
            c"OO|O".as_ptr(),
            ptr::addr_of_mut!(APP_INIT_KWLIST).cast(),
            &mut width_obj,
            &mut height_obj,
            &mut bridge,
        )
    } == 0
    {
        return -1;
    }
    let width = unsafe { PyLong_AsLong(width_obj) };
    let height = unsafe { PyLong_AsLong(height_obj) };
    if width <= 0 || height <= 0 {
        unsafe { set_error(PyExc_ValueError, "width and height must be positive") };
        return -1;
    }
    match EmbeddedServoApp::new(width as u32, height as u32) {
        Ok(app) => unsafe {
            (*self_).app = Box::into_raw(Box::new(app));
            (*self_).bridge = bridge;
            if !bridge.is_null() {
                Py_IncRef(bridge);
            }
            (*self_).closed = false;
            0
        },
        Err(e) => {
            unsafe { set_error(PyExc_RuntimeError, &e) };
            -1
        },
    }
}
unsafe extern "C" fn app_dealloc(self_: *mut PyAppObject) {
    unsafe {
        if !(*self_).app.is_null() {
            (*(*self_).app).trace(format_args!("app: dealloc"));
            (*(*self_).app).bridge_transport.clear_for_close();
            drop(Box::from_raw((*self_).app));
            (*self_).app = ptr::null_mut();
        }
        if !(*self_).bridge.is_null() {
            Py_DecRef((*self_).bridge);
        }
        (*self_).bridge = ptr::null_mut();
    }
}
unsafe fn get_app<'a>(self_: *mut PyAppObject) -> Result<&'a EmbeddedServoApp, ()> {
    unsafe {
        if (*self_).app.is_null() {
            set_error(PyExc_RuntimeError, "App is closed");
            Err(())
        } else {
            Ok(&*(*self_).app)
        }
    }
}
unsafe fn get_app_mut<'a>(self_: *mut PyAppObject) -> Result<&'a mut EmbeddedServoApp, ()> {
    unsafe {
        if (*self_).app.is_null() {
            set_error(PyExc_RuntimeError, "App is closed");
            Err(())
        } else {
            Ok(&mut *(*self_).app)
        }
    }
}
unsafe extern "C" fn app_load_path(self_: *mut PyAppObject, arg: *mut PyObject) -> *mut PyObject {
    let Ok(app) = (unsafe { get_app_mut(self_) }) else {
        return ptr::null_mut();
    };
    let raw = unsafe { PyUnicode_AsUTF8(arg) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    let path = unsafe { CStr::from_ptr(raw) }
        .to_string_lossy()
        .into_owned();
    let canonical = match PathBuf::from(path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            unsafe { set_error(PyExc_ValueError, &format!("failed to resolve path: {e}")) };
            return ptr::null_mut();
        },
    };
    let Some(file_name) = canonical.file_name().and_then(|n| n.to_str()) else {
        unsafe { set_error(PyExc_ValueError, "path must name a UTF-8 file") };
        return ptr::null_mut();
    };
    let package_root = canonical.parent().unwrap_or_else(|| Path::new("."));
    unsafe {
        std::env::set_var("SERVORENA_PACKAGE_ID", DEFAULT_PACKAGE_ID);
        std::env::set_var("SERVORENA_PACKAGE_ROOT", package_root);
    }
    let url = match Url::parse(&format!("asset://{DEFAULT_PACKAGE_ID}/{file_name}")) {
        Ok(u) => u,
        Err(e) => {
            unsafe { set_error(PyExc_ValueError, &format!("failed to build asset URL: {e}")) };
            return ptr::null_mut();
        },
    };
    app.trace(format_args!(
        "navigation: load source={} package_root={} url={url}",
        canonical.display(),
        package_root.display(),
    ));
    app.clear_bridge_for_navigation();
    app.webview.load(url);
    unsafe {
        Py_IncRef(py_none());
        py_none()
    }
}
unsafe extern "C" fn app_run(self_: *mut PyAppObject, _args: *mut PyObject) -> *mut PyObject {
    let mut logged_begin = false;
    while unsafe { !(*self_).closed } {
        let Ok(app) = (unsafe { get_app_mut(self_) }) else {
            break;
        };
        if !logged_begin {
            app.trace(format_args!("load: wait begin"));
            logged_begin = true;
        }
        app.spin_once();
        if app.presentation.closed_by_window {
            app.trace(format_args!("load: aborted window closed"));
            unsafe {
                (*self_).closed = true;
            }
            break;
        }
        if app.webview.load_status() == LoadStatus::Complete {
            app.trace(format_args!("load: complete"));
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    unsafe {
        Py_IncRef(py_none());
        py_none()
    }
}
unsafe extern "C" fn app_pump(self_: *mut PyAppObject, _args: *mut PyObject) -> *mut PyObject {
    let Ok(app) = (unsafe { get_app_mut(self_) }) else {
        return ptr::null_mut();
    };
    if let Err(error) = app.pump_once() {
        app.trace(format_args!("pump: failed error={error}"));
        unsafe { set_error(PyExc_RuntimeError, &error) };
        return ptr::null_mut();
    }
    if app.presentation.closed_by_window {
        app.trace(format_args!("pump: window closed"));
        unsafe {
            (*self_).closed = true;
        }
    }
    unsafe {
        Py_IncRef(py_none());
        py_none()
    }
}

unsafe extern "C" fn app_close(self_: *mut PyAppObject, _args: *mut PyObject) -> *mut PyObject {
    unsafe {
        (*self_).closed = true;
        if !(*self_).app.is_null() {
            (*(*self_).app).trace(format_args!("app: close"));
            (*(*self_).app).bridge_transport.clear_for_close();
            drop(Box::from_raw((*self_).app));
            (*self_).app = ptr::null_mut();
        }
        if !(*self_).bridge.is_null() {
            Py_DecRef((*self_).bridge);
        }
        (*self_).bridge = ptr::null_mut();
        Py_IncRef(py_none());
        py_none()
    }
}
unsafe fn unicode_to_string(object: *mut PyObject) -> Result<String, ()> {
    let raw = unsafe { PyUnicode_AsUTF8(object) };
    if raw.is_null() {
        Err(())
    } else {
        Ok(unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned())
    }
}

unsafe extern "C" fn app_read(self_: *mut PyAppObject, _args: *mut PyObject) -> *mut PyObject {
    let Ok(app) = (unsafe { get_app_mut(self_) }) else {
        return ptr::null_mut();
    };
    let Some(frame) = app.bridge_transport.read_for_python() else {
        unsafe {
            Py_IncRef(py_none());
            return py_none();
        }
    };
    app.trace(format_args!(
        "frame: read receipt={} json_bytes={}",
        frame.receipt,
        frame.json.len()
    ));

    let tuple = unsafe { PyTuple_New(2) };
    if tuple.is_null() {
        return ptr::null_mut();
    }
    let receipt = unsafe { PyLong_FromUnsignedLongLong(frame.receipt) };
    let json = unsafe {
        PyUnicode_FromStringAndSize(frame.json.as_ptr().cast(), frame.json.len() as isize)
    };
    if receipt.is_null() || json.is_null() {
        unsafe { Py_DecRef(tuple) };
        return ptr::null_mut();
    }
    if unsafe { PyTuple_SetItem(tuple, 0, receipt) } < 0 {
        unsafe {
            Py_DecRef(receipt);
            Py_DecRef(json);
            Py_DecRef(tuple);
        }
        return ptr::null_mut();
    }
    if unsafe { PyTuple_SetItem(tuple, 1, json) } < 0 {
        unsafe {
            Py_DecRef(json);
            Py_DecRef(tuple);
        }
        return ptr::null_mut();
    }
    tuple
}

unsafe extern "C" fn app_write(self_: *mut PyAppObject, args: *mut PyObject) -> *mut PyObject {
    let Ok(app) = (unsafe { get_app_mut(self_) }) else {
        return ptr::null_mut();
    };
    let mut receipt_obj: *mut PyObject = ptr::null_mut();
    let mut json_obj: *mut PyObject = ptr::null_mut();
    if unsafe { PyArg_ParseTuple(args, c"OO".as_ptr(), &mut receipt_obj, &mut json_obj) } == 0 {
        return ptr::null_mut();
    }

    let receipt = unsafe { PyLong_AsUnsignedLongLong(receipt_obj) };
    let Ok(json) = (unsafe { unicode_to_string(json_obj) }) else {
        return ptr::null_mut();
    };
    app.trace(format_args!(
        "reply: write receipt={receipt} json_bytes={}",
        json.len()
    ));
    let script = match app.bridge_transport.prepare_reply(receipt, &json) {
        Ok(script) => script,
        Err(error) => {
            app.trace(format_args!("reply: reject receipt={receipt} error={error}"));
            unsafe { set_error(PyExc_RuntimeError, &error) };
            return ptr::null_mut();
        },
    };
    app.schedule_reply_delivery(script);
    unsafe {
        Py_IncRef(py_none());
        py_none()
    }
}

unsafe fn py_none() -> *mut PyObject {
    ptr::addr_of_mut!(_Py_NoneStruct)
}
static mut APP_INIT_KWLIST: [*mut c_char; 4] = [
    c"width".as_ptr().cast_mut(),
    c"height".as_ptr().cast_mut(),
    c"bridge".as_ptr().cast_mut(),
    ptr::null_mut(),
];
static mut APP_METHODS: [PyMethodDef; 7] = [
    PyMethodDef {
        ml_name: c"load_path".as_ptr(),
        ml_meth: app_load_path as *mut c_void,
        ml_flags: METH_O,
        ml_doc: c"Load a local package entry path.".as_ptr(),
    },
    PyMethodDef {
        ml_name: c"run".as_ptr(),
        ml_meth: app_run as *mut c_void,
        ml_flags: METH_NOARGS,
        ml_doc: c"Run the Servo event loop.".as_ptr(),
    },
    PyMethodDef {
        ml_name: c"close".as_ptr(),
        ml_meth: app_close as *mut c_void,
        ml_flags: METH_NOARGS,
        ml_doc: c"Close the embedded Servo instance.".as_ptr(),
    },
    PyMethodDef {
        ml_name: c"pump".as_ptr(),
        ml_meth: app_pump as *mut c_void,
        ml_flags: METH_NOARGS,
        ml_doc: c"Run one bounded Servo owner-thread pump turn and bridge delivery pass.".as_ptr(),
    },
    PyMethodDef {
        ml_name: c"write".as_ptr(),
        ml_meth: app_write as *mut c_void,
        ml_flags: METH_VARARGS,
        ml_doc: c"Write an opaque JSON frame against a private transport receipt.".as_ptr(),
    },
    PyMethodDef {
        ml_name: c"read".as_ptr(),
        ml_meth: app_read as *mut c_void,
        ml_flags: METH_NOARGS,
        ml_doc: c"Read the next opaque JSON bridge frame and private receipt, if any.".as_ptr(),
    },
    PyMethodDef {
        ml_name: ptr::null(),
        ml_meth: ptr::null_mut(),
        ml_flags: 0,
        ml_doc: ptr::null(),
    },
];
static mut APP_SLOTS: [PyType_Slot; 5] = [
    PyType_Slot {
        slot: PY_TP_NEW,
        pfunc: PyType_GenericNew as *mut c_void,
    },
    PyType_Slot {
        slot: PY_TP_INIT,
        pfunc: app_init as *mut c_void,
    },
    PyType_Slot {
        slot: PY_TP_DEALLOC,
        pfunc: app_dealloc as *mut c_void,
    },
    PyType_Slot {
        slot: PY_TP_METHODS,
        pfunc: ptr::addr_of_mut!(APP_METHODS) as *mut c_void,
    },
    PyType_Slot {
        slot: 0,
        pfunc: ptr::null_mut(),
    },
];
static mut APP_SPEC: PyType_Spec = PyType_Spec {
    name: c"severin.App".as_ptr(),
    basicsize: std::mem::size_of::<PyAppObject>() as c_int,
    itemsize: 0,
    flags: PY_TPFLAGS_DEFAULT | PY_TPFLAGS_BASETYPE,
    slots: ptr::addr_of_mut!(APP_SLOTS).cast::<PyType_Slot>(),
};
static mut MODULE_SLOTS: [PyModuleDef_Slot; 2] = [
    PyModuleDef_Slot {
        slot: PY_MOD_EXEC,
        value: module_exec as *mut c_void,
    },
    PyModuleDef_Slot {
        slot: 0,
        value: ptr::null_mut(),
    },
];
static mut MODULE_DEF: PyModuleDef = PyModuleDef {
    base: PyModuleDef_Base {
        ob_base: [0; 2],
        init: ptr::null_mut(),
        index: 0,
        copy: ptr::null_mut(),
    },
    name: c"severin".as_ptr(),
    doc: c"Experimental in-process Servo Python embedding.".as_ptr(),
    size: 0,
    methods: ptr::null_mut(),
    slots: ptr::addr_of_mut!(MODULE_SLOTS).cast::<PyModuleDef_Slot>(),
    traverse: ptr::null_mut(),
    clear: ptr::null_mut(),
    free: ptr::null_mut(),
};

unsafe extern "C" fn module_exec(module: *mut PyObject) -> c_int {
    let app_type = unsafe { PyType_FromSpec(ptr::addr_of_mut!(APP_SPEC)) };
    if app_type.is_null() {
        return -1;
    }
    if unsafe { PyModule_AddObject(module, c"App".as_ptr(), app_type) } < 0 {
        unsafe { Py_DecRef(app_type) };
        return -1;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn PyInit_severin() -> *mut PyObject {
    unsafe { PyModuleDef_Init(ptr::addr_of_mut!(MODULE_DEF)) }
}
