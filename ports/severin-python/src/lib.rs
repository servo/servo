/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Experimental in-process CPython bindings for the Severin local runtime.

use std::collections::{HashSet, VecDeque};
use std::ffi::{CStr, CString, c_char, c_int, c_long, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use dpi::PhysicalSize;
use servo::{
    EventLoopWaker, LoadStatus, Preferences, RenderingContext, Servo, ServoBuilder, ServoUrl,
    SoftwareRenderingContext, WebView, WebViewBuilder,
};
use url::Url;

const DEFAULT_PACKAGE_ID: &str = "com.example.app";
const PY_TPFLAGS_DEFAULT: u64 = 0;
const PY_TPFLAGS_BASETYPE: u64 = 1 << 10;
const PY_MOD_EXEC: c_int = 0;
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
    fn Py_DecRef(object: *mut PyObject);
    fn Py_XIncRef(object: *mut PyObject);
    fn Py_XDecRef(object: *mut PyObject);
    fn PyLong_AsLong(object: *mut PyObject) -> c_long;
    fn PyLong_AsUnsignedLongLong(object: *mut PyObject) -> u64;
    fn PyLong_FromUnsignedLongLong(value: u64) -> *mut PyObject;
    fn PyTuple_New(size: isize) -> *mut PyObject;
    fn PyTuple_SetItem(tuple: *mut PyObject, position: isize, item: *mut PyObject) -> c_int;
    fn PyUnicode_FromStringAndSize(string: *const c_char, size: isize) -> *mut PyObject;
    fn PyUnicode_AsUTF8(object: *mut PyObject) -> *const c_char;
    static mut _Py_NoneStruct: PyObject;
}

#[derive(Clone)]
struct PythonEventLoopWaker(Arc<AtomicBool>);
impl EventLoopWaker for PythonEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }
    fn wake(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

struct EmbeddedServoApp {
    servo: Servo,
    webview: WebView,
    _rendering_context: Rc<dyn RenderingContext>,
    wake_flag: Arc<AtomicBool>,
    bridge_transport: BridgeTransport,
}

#[derive(Default)]
struct BridgeTransport {
    next_receipt: u64,
    inbound: VecDeque<BridgeFrame>,
    pending: HashSet<u64>,
}

struct BridgeFrame {
    receipt: u64,
    json: String,
}

impl BridgeTransport {
    #[allow(dead_code)]
    fn enqueue_from_javascript(&mut self, json: String) -> Result<u64, String> {
        validate_json_frame(&json)?;
        self.next_receipt = self.next_receipt.saturating_add(1);
        let receipt = self.next_receipt;
        self.pending.insert(receipt);
        self.inbound.push_back(BridgeFrame { receipt, json });
        Ok(receipt)
    }

    fn read_for_python(&mut self) -> Option<BridgeFrame> {
        self.inbound.pop_front()
    }

    fn write_from_python(&mut self, receipt: u64, json: &str) -> Result<(), String> {
        validate_json_frame(json)?;
        if !self.pending.remove(&receipt) {
            return Err("bridge reply target no longer exists".to_owned());
        }

        // TODO: Resolve the matching JavaScript Promise when the JS shim is wired
        // into Servo script. The native layer deliberately treats this as an
        // opaque JSON frame and private transport receipt only.
        Ok(())
    }
}

fn validate_json_frame(json: &str) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(json)
        .map(|_| ())
        .map_err(|error| format!("invalid JSON bridge frame: {error}"))
}
impl EmbeddedServoApp {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let rendering_context = Rc::new(
            SoftwareRenderingContext::new(PhysicalSize { width, height })
                .map_err(|e| format!("failed to create rendering context: {e:?}"))?,
        );
        rendering_context
            .make_current()
            .map_err(|e| format!("failed to make rendering context current: {e:?}"))?;
        let wake_flag = Arc::new(AtomicBool::new(false));
        let mut preferences = Preferences::default();
        preferences.network_http_proxy_uri = String::new();
        preferences.network_https_proxy_uri = String::new();
        let servo = ServoBuilder::default()
            .preferences(preferences)
            .event_loop_waker(Box::new(PythonEventLoopWaker(wake_flag.clone())))
            .build();
        let webview = WebViewBuilder::new(&servo, rendering_context.clone()).build();
        Ok(Self {
            servo,
            webview,
            _rendering_context: rendering_context,
            wake_flag,
            bridge_transport: BridgeTransport::default(),
        })
    }
    fn spin_once(&self) {
        self.wake_flag.store(false, Ordering::Relaxed);
        self.servo.spin_event_loop();
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
            Py_XIncRef(bridge);
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
            drop(Box::from_raw((*self_).app));
            (*self_).app = ptr::null_mut();
        }
        Py_XDecRef((*self_).bridge);
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
    let Ok(app) = (unsafe { get_app(self_) }) else {
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
    app.webview.load(ServoUrl::from_url(url));
    unsafe {
        Py_XIncRef(py_none());
        py_none()
    }
}
unsafe extern "C" fn app_run(self_: *mut PyAppObject, _args: *mut PyObject) -> *mut PyObject {
    while unsafe { !(*self_).closed } {
        let Ok(app) = (unsafe { get_app(self_) }) else {
            break;
        };
        app.spin_once();
        if app.webview.load_status() == LoadStatus::Complete {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    unsafe {
        Py_XIncRef(py_none());
        py_none()
    }
}
unsafe extern "C" fn app_close(self_: *mut PyAppObject, _args: *mut PyObject) -> *mut PyObject {
    unsafe {
        (*self_).closed = true;
        if !(*self_).app.is_null() {
            drop(Box::from_raw((*self_).app));
            (*self_).app = ptr::null_mut();
        }
        Py_XDecRef((*self_).bridge);
        (*self_).bridge = ptr::null_mut();
        Py_XIncRef(py_none());
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
            Py_XIncRef(py_none());
            return py_none();
        }
    };

    let tuple = unsafe { PyTuple_New(2) };
    if tuple.is_null() {
        return ptr::null_mut();
    }
    let receipt = unsafe { PyLong_FromUnsignedLongLong(frame.receipt) };
    let json = unsafe {
        PyUnicode_FromStringAndSize(frame.json.as_ptr().cast(), frame.json.len() as isize)
    };
    if receipt.is_null() || json.is_null() {
        unsafe { Py_XDecRef(tuple) };
        return ptr::null_mut();
    }
    if unsafe { PyTuple_SetItem(tuple, 0, receipt) } < 0 {
        unsafe {
            Py_XDecRef(receipt);
            Py_XDecRef(json);
            Py_XDecRef(tuple);
        }
        return ptr::null_mut();
    }
    if unsafe { PyTuple_SetItem(tuple, 1, json) } < 0 {
        unsafe {
            Py_XDecRef(json);
            Py_XDecRef(tuple);
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
    if let Err(error) = app.bridge_transport.write_from_python(receipt, &json) {
        unsafe { set_error(PyExc_RuntimeError, &error) };
        return ptr::null_mut();
    }
    unsafe {
        Py_XIncRef(py_none());
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
static mut APP_METHODS: [PyMethodDef; 6] = [
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
    slots: ptr::addr_of_mut!(APP_SLOTS),
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
    slots: ptr::addr_of_mut!(MODULE_SLOTS),
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
