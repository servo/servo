/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using gonk interfaces.

use compositing::compositor_task::{self, CompositorProxy, CompositorReceiver};
use compositing::windowing::{WindowEvent, WindowMethods};
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use gleam::gl;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeDisplay;
use libc::c_int;
use msg::constellation_msg::{Key, KeyModifiers};
use net_traits::net_error_list::NetError;
use std::ffi::CString;
use std::mem::{transmute, size_of, zeroed};
use std::ptr;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender, Receiver};
use url::Url;
use util::cursor::Cursor;
use util::geometry::ScreenPx;

use egl::egl;
use egl::egl::EGLConfig;
use egl::egl::EGLContext;
use egl::egl::EGLDisplay;
use egl::egl::EGLSurface;
use egl::egl::EGLint;

use libc::c_char;
use libc::c_void;
use libc::close;
use libc::size_t;

const GRALLOC_USAGE_HW_TEXTURE: c_int = 0x00000100;
const GRALLOC_USAGE_HW_RENDER: c_int = 0x00000200;
const GRALLOC_USAGE_HW_2D: c_int = 0x00000400;
const GRALLOC_USAGE_HW_COMPOSER: c_int = 0x00000800;
const GRALLOC_USAGE_HW_FB: c_int = 0x00001000;

// system/core/include/cutils/native_handle.h

#[repr(C)]
pub struct native_handle {
    version: c_int,
    numFds: c_int,
    numInts: c_int,
    data: [c_int; 0],
}

// system/core/include/system/window.h

#[repr(C)]
pub struct ANativeBase {
    magic: u32,
    version: u32,
    reserved: [isize; 4],
    incRef: extern fn(*mut ANativeBase),
    decRef: extern fn(*mut ANativeBase),
}

#[repr(C)]
pub struct ANativeWindowBuffer {
    common: ANativeBase,
    width: c_int,
    height: c_int,
    stride: c_int,
    format: c_int,
    usage: c_int,
    reserved: [*mut c_void; 2],
    handle: *const native_handle,
    reserved_proc: [*mut c_void; 8],
}

#[repr(C)]
pub struct ANativeWindow {
    common: ANativeBase,
    flags: u32,
    minSwapInterval: c_int,
    maxSwapInterval: c_int,
    xdpi: f32,
    ydpi: f32,
    oem: [isize; 4],
    setSwapInterval: extern fn(*mut ANativeWindow, c_int) -> c_int,
    //dequeueBuffer_DEPRECATED: extern fn(*mut ANativeWindow, *mut *mut ANativeWindowBuffer) -> c_int,
    //lockBuffer_DEPRECATED: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer) -> c_int,
    //queueBuffer_DEPRECATED: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer) -> c_int,
    dequeueBuffer_DEPRECATED: *const c_void,
    lockBuffer_DEPRECATED: *const c_void,
    queueBuffer_DEPRECATED: *const c_void,
    query: extern fn(*const ANativeWindow, c_int, *mut c_int) -> c_int,
    perform: extern fn(*mut ANativeWindow, c_int, ...) -> c_int,
    //cancelBuffer_DEPRECATED: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer) -> c_int,
    cancelBuffer_DEPRECATED: *const c_void,
    dequeueBuffer: extern fn(*mut ANativeWindow, *mut *mut ANativeWindowBuffer, *mut c_int) -> c_int,
    queueBuffer: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer, c_int) -> c_int,
    cancelBuffer: extern fn(*mut ANativeWindow, *mut ANativeWindowBuffer, c_int) -> c_int,
}

// hardware/libhardware/include/hardware/hardware.h

#[repr(C)]
pub struct hw_module_methods {
    open: extern fn(*const hw_module, *const c_char, *mut *const hw_device) -> c_int,
}

#[repr(C)]
pub struct hw_module {
    tag: u32,
    module_api_version: u16,
    hal_api_version: u16,
    id: *const c_char,
    name: *const c_char,
    author: *const c_char,
    methods: *mut hw_module_methods,
    dso: *mut u32,
    reserved: [u32; (32-7)],
}

#[repr(C)]
pub struct hw_device {
    tag: u32,
    version: u32,
    module: *mut hw_module,
    reserved: [u32; 12],
    close: extern fn(*mut hw_device) -> c_int,
}

#[link(name = "hardware")]
extern {
    fn hw_get_module(id: *const c_char, module: *mut *const hw_module) -> c_int;
}

// hardware/libhardware/include/hardware/hwcomposer.h

#[repr(C)]
pub struct hwc_color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct hwc_rect {
    left: c_int,
    top: c_int,
    right: c_int,
    bottom: c_int,
}

#[repr(C)]
pub struct hwc_region {
    numRects: i32,
    rects: *const hwc_rect,
}

const HWC_FRAMEBUFFER: i32 = 0;
const HWC_OVERLAY: i32 = 1;
const HWC_BACKGROUND: i32 = 2;
const HWC_FRAMEBUFFER_TARGET: i32 = 3;
const HWC_BLIT: i32 = 4;

const HWC_SKIP_LAYER: u32 = 1;

#[repr(C)]
pub struct hwc_layer {
    compositionType: i32,
    hints: u32,
    flags: u32,
    handle: *const native_handle,
    transform: u32,
    blending: i32,
    sourceCrop: hwc_rect, // If HWC 1.3, then this takes floats
    displayFrame: hwc_rect,
    visibleRegionScreen: hwc_region,
    acquireFenceFd: c_int,
    releaseFenceFd: c_int,
    planeAlpha: u8,
    pad: [u8; 3],
    reserved: [i32; (24 - 19)],
}

#[repr(C)]
pub struct hwc_display_contents {
    retireFenceFd: c_int,
    // HWC 1.0 not supported
    outbuf: *const u32,
    outbufAcquireFenceFd: c_int,
    flags: u32,
    numHwLayers: size_t,
    hwLayers: [hwc_layer; 2],
}

#[repr(C)]
pub struct hwc_procs {
    invalidate: extern fn(*const hwc_procs),
    vsync: extern fn(*const hwc_procs, c_int, i64),
    hotplug: extern fn(*const hwc_procs, c_int, c_int),
}

const HWC_DISPLAY_NO_ATTRIBUTE: u32 = 0;
const HWC_DISPLAY_VSYNC_PERIOD: u32 = 1;
const HWC_DISPLAY_WIDTH: u32 = 2;
const HWC_DISPLAY_HEIGHT: u32 = 3;
const HWC_DISPLAY_DPI_X: u32 = 4;
const HWC_DISPLAY_DPI_Y: u32 = 5;

#[repr(C)]
pub struct hwc_composer_device {
    common: hw_device,
    prepare: extern fn(*mut hwc_composer_device, size_t, *mut *mut hwc_display_contents) -> c_int,
    set: extern fn(*mut hwc_composer_device, size_t, *mut *mut hwc_display_contents) -> c_int,
    eventControl: extern fn(*mut hwc_composer_device, c_int, c_int, c_int) -> c_int,
    blank: extern fn(*mut hwc_composer_device, c_int, c_int) -> c_int,
    query: extern fn(*mut hwc_composer_device, c_int, *mut c_int) -> c_int,
    registerProcs: extern fn(*mut hwc_composer_device, *const hwc_procs),
    dump: extern fn(*mut hwc_composer_device, *const c_char, c_int),
    getDisplayConfigs: extern fn(*mut hwc_composer_device, c_int, *mut u32, *mut size_t) -> c_int,
    getDisplayAttributes: extern fn(*mut hwc_composer_device, c_int, u32, *const u32, *mut i32) -> c_int,
    reserved: [*mut c_void; 4],
}

// system/core/include/system/graphics.h

#[repr(C)]
pub struct android_ycbcr {
    y: *mut c_void,
    cb: *mut c_void,
    cr: *mut c_void,
    ystride: size_t,
    cstride: size_t,
    chroma_step: size_t,
    reserved: [u32; 8],
}

// hardware/libhardware/include/hardware/gralloc.h

#[repr(C)]
pub struct gralloc_module {
    common: hw_module,
    registerBuffer: extern fn(*const gralloc_module, *const native_handle) -> c_int,
    unregisterBuffer: extern fn(*const gralloc_module, *const native_handle) -> c_int,
    lock: extern fn(*const gralloc_module, *const native_handle, c_int, c_int, c_int, c_int,
                    *mut *mut c_void) -> c_int,
    unlock: extern fn(*const gralloc_module, *const native_handle) -> c_int,
    perform: extern fn(*const gralloc_module, c_int, ...) -> c_int,
    lock_ycbcr: extern fn(*const gralloc_module, *const native_handle, c_int, c_int, c_int, c_int,
                          c_int, *mut android_ycbcr) -> c_int,
    reserved: [*mut c_void; 6],
}

#[repr(C)]
pub struct alloc_device {
    common: hw_device,
    allocSize: extern fn(*mut alloc_device, c_int, c_int, c_int, c_int, *mut *const native_handle,
                         *mut c_int, c_int) -> c_int,
    alloc: extern fn(*mut alloc_device, c_int, c_int, c_int, c_int, *mut *const native_handle,
                     *mut c_int) -> c_int,
    free: extern fn(*mut alloc_device, *const native_handle) -> c_int,
    dump: Option<extern fn(*mut alloc_device, *mut c_char, c_int)>,
    reserved: [*mut c_void; 7],
}

#[repr(C)]
pub struct GonkNativeWindow {
    window: ANativeWindow,
    set_usage: extern fn(*mut GonkNativeWindow, c_int) -> c_int,
    set_format: extern fn(*mut GonkNativeWindow, c_int) -> c_int,
    set_transform: extern fn(*mut GonkNativeWindow, c_int) -> c_int,
    set_dimensions: extern fn(*mut GonkNativeWindow, c_int, c_int) -> c_int,
    api_connect: extern fn(*mut GonkNativeWindow, c_int) -> c_int,
    api_disconnect: extern fn(*mut GonkNativeWindow, c_int) -> c_int,
    count: i32,
    alloc_dev: *mut alloc_device,
    hwc_dev: *mut hwc_composer_device,
    width: i32,
    height: i32,
    format: c_int,
    usage: c_int,
    last_fence: c_int,
    last_idx: i32,
    bufs: [Option<*mut GonkNativeWindowBuffer>; 2],
    fences: [c_int; 2],
}

impl ANativeBase {
    fn magic(a: char, b: char, c: char, d: char) -> u32 {
        (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | d as u32
    }
}

#[repr(C)]
pub struct GonkNativeWindowBuffer {
    buffer: ANativeWindowBuffer,
    count: i32,
}

#[link(name = "native_window_glue", kind = "static")]
extern {
    fn gnw_perform(win: *mut ANativeWindow, op: c_int, ...) -> c_int;
}

#[link(name = "suspend")]
extern {
    fn autosuspend_disable();
}

#[allow(unused_variables)]
extern fn setSwapInterval(base: *mut ANativeWindow,
                          interval: c_int) -> c_int {
    0
}

const NATIVE_WINDOW_WIDTH: c_int = 0;
const NATIVE_WINDOW_HEIGHT: c_int = 1;
const NATIVE_WINDOW_FORMAT: c_int = 2;
const NATIVE_WINDOW_DEFAULT_WIDTH: c_int = 6;
const NATIVE_WINDOW_DEFAULT_HEIGHT: c_int = 7;
const NATIVE_WINDOW_TRANSFORM_HINT: c_int = 8;

extern fn query(base: *const ANativeWindow,
                what: c_int, value: *mut c_int) -> c_int {
    unsafe {
        let window: &GonkNativeWindow = transmute(base);
        match what {
            NATIVE_WINDOW_WIDTH => { *value = window.width; 0 }
            NATIVE_WINDOW_HEIGHT => { *value = window.height; 0 }
            NATIVE_WINDOW_FORMAT => { *value = window.format; 0 }
            NATIVE_WINDOW_DEFAULT_WIDTH => { *value = window.width; 0 }
            NATIVE_WINDOW_DEFAULT_HEIGHT => { *value = window.height; 0 }
            NATIVE_WINDOW_TRANSFORM_HINT => { *value = 0; 0 }
            _ => { println!("Unsupported query - {}", what); -1 }
        }
    }
}

extern fn dequeueBuffer(base: *mut ANativeWindow, buf: *mut *mut ANativeWindowBuffer, fence: *mut c_int) -> c_int {
    unsafe {
        let window: &mut GonkNativeWindow = transmute(base);
        for idx in 0..window.bufs.len() {
            if idx == window.last_idx as usize {
                continue;
            }
            match window.bufs[idx] {
                Some(entry) => {
                    (*buf) = transmute(entry);
                    window.bufs[idx] = None;
                    *fence = window.fences[idx];
                    window.fences[idx] = -1;
                    return 0;
                },
                None => (),
            }
        }
    }
    -1
}

extern fn queueBuffer(base: *mut ANativeWindow, buf: *mut ANativeWindowBuffer, fence: c_int) -> c_int {
    unsafe {
        let window: &mut GonkNativeWindow = transmute(base);
        for idx in 0..window.bufs.len() {
            match window.bufs[idx] {
                Some(_) => (),
                None => {
                    window.last_idx = idx as i32;
                    window.bufs[idx] = Some(transmute(buf));
                    window.fences[idx] = window.draw(buf, fence);
                    return 0;
                },
            }
        }
    }
    -1
}

extern fn cancelBuffer(base: *mut ANativeWindow, buf: *mut ANativeWindowBuffer, fence: c_int) -> c_int {
    unsafe {
        let window: &mut GonkNativeWindow = transmute(base);
        for idx in 0..window.bufs.len() {
            match window.bufs[idx] {
                Some(_) => (),
                None => {
                    window.bufs[idx] = Some(transmute(buf));
                    window.fences[idx] = -1;
                    close(fence);
                    return 0;
                },
            }
        }
    }
    -1
}

extern fn set_usage(window: *mut GonkNativeWindow,
                    usage: c_int) -> c_int {
    println!("Setting usage flags to {}", usage);
    unsafe {
        (*window).usage = usage;
        (*window).bufs[0] = Some(GonkNativeWindowBuffer::new(
            (*window).alloc_dev, (*window).width, (*window).height,
            (*window).format, (*window).usage));
        (*window).bufs[1] = Some(GonkNativeWindowBuffer::new(
            (*window).alloc_dev, (*window).width, (*window).height,
            (*window).format, (*window).usage));
    }
    0
}

extern fn set_format(window: *mut GonkNativeWindow,
                     format: c_int) -> c_int {
    println!("Setting format to {}", format);
    unsafe {
        (*window).format = format;
    }
    0
}

extern fn set_transform(_: *mut GonkNativeWindow,
                        _: c_int) -> c_int {
    0
}

extern fn set_dimensions(_: *mut GonkNativeWindow,
                         _: c_int, _: c_int) -> c_int {
    0
}

#[allow(unused_variables)]
extern fn api_connect(window: *mut GonkNativeWindow,
                      api: c_int) -> c_int {
    0
}

#[allow(unused_variables)]
extern fn api_disconnect(window: *mut GonkNativeWindow,
                         api: c_int) -> c_int {
    0
}

extern fn gnw_incRef(base: *mut ANativeBase) {
    let win: &mut GonkNativeWindow = unsafe { transmute(base) };
    win.count += 1;
}

extern fn gnw_decRef(base: *mut ANativeBase) {
    let win: &mut GonkNativeWindow = unsafe { transmute(base) };
    win.count -= 1;
    if win.count == 0 {
        unsafe { transmute::<_, Box<GonkNativeWindow>>(base) };
    }
}

impl GonkNativeWindow {
    pub fn new(alloc_dev: *mut alloc_device, hwc_dev: *mut hwc_composer_device, width: i32,
               height: i32, usage: c_int) -> *mut GonkNativeWindow {
        let win = box GonkNativeWindow {
            window: ANativeWindow {
                common: ANativeBase {
                    magic: ANativeBase::magic('_', 'w', 'n', 'd'),
                    version: size_of::<ANativeBase>() as u32,
                    reserved: unsafe { zeroed() },
                    incRef: gnw_incRef,
                    decRef: gnw_decRef,
                },
                flags: 0,
                minSwapInterval: 0,
                maxSwapInterval: 0,
                xdpi: 0f32,
                ydpi: 0f32,
                oem: unsafe { zeroed() },
                setSwapInterval: setSwapInterval,
                dequeueBuffer_DEPRECATED: ptr::null(),
                lockBuffer_DEPRECATED: ptr::null(),
                queueBuffer_DEPRECATED: ptr::null(),
                query: query,
                perform: unsafe { transmute(gnw_perform) },
                cancelBuffer_DEPRECATED: ptr::null(),
                dequeueBuffer: dequeueBuffer,
                queueBuffer: queueBuffer,
                cancelBuffer: cancelBuffer,
            },
            set_usage: set_usage,
            set_format: set_format,
            set_transform: set_transform,
            set_dimensions: set_dimensions,
            api_connect: api_connect,
            api_disconnect: api_disconnect,
            count: 1,
            alloc_dev: alloc_dev,
            hwc_dev: hwc_dev,
            width: width,
            height: height,
            format: 0,
            usage: usage,
            last_fence: -1,
            last_idx: -1,
            bufs: unsafe { zeroed() },
            fences: [-1, -1],
        };

        unsafe { transmute(win) }
    }

    fn draw(&mut self, buf: *mut ANativeWindowBuffer, fence: c_int) -> c_int {
        let gonkbuf: &mut GonkNativeWindowBuffer = unsafe { transmute(buf) };
        let rect = hwc_rect {
            left: 0, top: 0, right: gonkbuf.buffer.width, bottom: gonkbuf.buffer.height
        };
        let mut list = hwc_display_contents {
            retireFenceFd: -1,
            outbuf: ptr::null(),
            outbufAcquireFenceFd: -1,
            flags: 1, /* HWC_GEOMETRY_CHANGED */
            numHwLayers: 2,
            hwLayers: [
                hwc_layer {
                    compositionType: HWC_FRAMEBUFFER,
                    hints: 0,
                    flags: HWC_SKIP_LAYER,
                    handle: ptr::null(),
                    transform: 0,
                    blending: 0,
                    sourceCrop: hwc_rect {
                        left: 0, top: 0, right: 0, bottom: 0
                    },
                    displayFrame: hwc_rect {
                        left: 0, top: 0, right: 0, bottom: 0
                    },
                    visibleRegionScreen: hwc_region {
                        numRects: 0,
                        rects: ptr::null(),
                    },
                    acquireFenceFd: -1,
                    releaseFenceFd: -1,
                    planeAlpha: 0xFF,
                    pad: [0, 0, 0],
                    reserved: [0, 0, 0, 0, 0],
                },
                hwc_layer {
                    compositionType: HWC_FRAMEBUFFER_TARGET,
                    hints: 0,
                    flags: 0,
                    handle: gonkbuf.buffer.handle,
                    transform: 0,
                    blending: 0,
                    sourceCrop: rect,
                    displayFrame: rect,
                    visibleRegionScreen: hwc_region {
                        numRects: 1,
                        rects: &rect,
                    },
                    acquireFenceFd: fence,
                    releaseFenceFd: -1,
                    planeAlpha: 0xFF,
                    pad: [0, 0, 0],
                    reserved: [0, 0, 0, 0, 0],
                },
            ],
        };
        unsafe {
            let mut displays: [*mut hwc_display_contents; 3] = [ &mut list, ptr::null_mut(), ptr::null_mut(), ];
            let _ = ((*self.hwc_dev).prepare)(self.hwc_dev,
                                              displays.len() as size_t,
                                              transmute(displays.as_mut_ptr()));
            let _ = ((*self.hwc_dev).set)(self.hwc_dev, displays.len() as size_t, transmute(displays.as_mut_ptr()));
            if list.retireFenceFd >= 0 {
                close(list.retireFenceFd);
            }
        }
        list.hwLayers[1].releaseFenceFd
    }
}

extern fn gnwb_incRef(base: *mut ANativeBase) {
    let buf: &mut GonkNativeWindowBuffer = unsafe { transmute(base) };
    buf.count += 1;
}

extern fn gnwb_decRef(base: *mut ANativeBase) {
    let buf: &mut GonkNativeWindowBuffer = unsafe { transmute(base) };
    buf.count -= 1;
    if buf.count == 0 {
        unsafe { transmute::<_, Box<GonkNativeWindowBuffer>>(base) };
    }
}

impl GonkNativeWindowBuffer {
    pub fn new(dev: *mut alloc_device,
               width: i32,
               height: i32,
               format: c_int, usage: c_int) -> *mut GonkNativeWindowBuffer {
        let mut buf = box GonkNativeWindowBuffer {
            buffer: ANativeWindowBuffer {
                common: ANativeBase {
                    magic: ANativeBase::magic('_', 'b', 'f', 'r'),
                    version: size_of::<ANativeBase>() as u32,
                    reserved: unsafe { zeroed() },
                    incRef: gnwb_incRef,
                    decRef: gnwb_decRef,
                },
                width: width,
                height: height,
                stride: 0,
                format: format,
                usage: usage,
                reserved: unsafe { zeroed() },
                handle: ptr::null(),
                reserved_proc: unsafe { zeroed() },
            },
            count: 1,
        };

        let ret = unsafe {
            ((*dev).alloc)(dev, width, height, format, usage,
                           &mut buf.buffer.handle, &mut buf.buffer.stride)
        };
        assert!(ret == 0, "Failed to allocate gralloc buffer!");

        unsafe { transmute(buf) }
    }
}

/// The type of a window.
pub struct Window {
    event_recv: Receiver<WindowEvent>,
    pub event_send: Sender<WindowEvent>,
    width: i32,
    height: i32,
    native_window: *mut GonkNativeWindow,
    dpy: EGLDisplay,
    ctx: EGLContext,
    surf: EGLSurface,
}

impl Window {
    /// Creates a new window.
    pub fn new() -> Rc<Window> {
        let mut hwc_mod = ptr::null();
        unsafe {
            let cstr = CString::new("hwcomposer").unwrap();
            let ret = hw_get_module(cstr.as_ptr(), &mut hwc_mod);
            assert!(ret == 0, "Failed to get HWC module!");
        }

        let hwc_device: *mut hwc_composer_device;
        unsafe {
            let mut device = ptr::null();
            let cstr = CString::new("composer").unwrap();
            let ret = ((*(*hwc_mod).methods).open)(hwc_mod, cstr.as_ptr(), &mut device);
            assert!(ret == 0, "Failed to get HWC device!");
            hwc_device = transmute(device);
            // Require HWC 1.1 or newer
            // XXX add HAL version function/macro
            assert!((*hwc_device).common.version > (1 << 8), "HWC too old!");
        }

        let attrs: [u32; 4] = [
            HWC_DISPLAY_WIDTH,
            HWC_DISPLAY_HEIGHT,
            HWC_DISPLAY_DPI_X,
            HWC_DISPLAY_NO_ATTRIBUTE];
        let mut values: [i32; 4] = [0, 0, 0, 0];
        unsafe {
            // In theory, we should check the return code.
            // However, there are HALs which implement this wrong.
            let _ = ((*hwc_device).getDisplayAttributes)(hwc_device, 0, 0, attrs.as_ptr(), values.as_mut_ptr());
        }

        let mut gralloc_mod = ptr::null();
        let alloc_dev: *mut alloc_device;
        unsafe {
            let mut device = ptr::null();
            let cstr = CString::new("gralloc").unwrap();
            let ret1 = hw_get_module(cstr.as_ptr(), &mut gralloc_mod);
            assert!(ret1 == 0, "Failed to get gralloc moudle!");
            let cstr2 = CString::new("gpu0").unwrap();
            let ret2 = ((*(*gralloc_mod).methods).open)(gralloc_mod, cstr2.as_ptr(), &mut device);
            assert!(ret2 == 0, "Failed to get gralloc moudle!");
            alloc_dev = transmute(device);
        }

        let width = values[0];
        let height = values[1];
        let dpy = egl::GetDisplay(unsafe { transmute(egl::EGL_DEFAULT_DISPLAY) });

        let ret1 = {
            let mut major: i32 = 0;
            let mut minor: i32 = 0;
            egl::Initialize(dpy, &mut major, &mut minor)
        };

        assert!(ret1 == 1, "Failed to initialize EGL!");

        let conf_attr =
            [egl::EGL_SURFACE_TYPE, egl::EGL_WINDOW_BIT,
             egl::EGL_RENDERABLE_TYPE, egl::EGL_OPENGL_ES2_BIT,
             egl::EGL_RED_SIZE, 8,
             egl::EGL_GREEN_SIZE, 8,
             egl::EGL_BLUE_SIZE, 8,
             egl::EGL_ALPHA_SIZE, 0,
             egl::EGL_NONE, 0];

        let mut config: EGLConfig = unsafe { transmute(0isize) };
        let mut num_config: EGLint = 0;

        let ret2 = unsafe {
            egl::ChooseConfig(dpy, transmute(conf_attr.as_ptr()), &mut config, 1, &mut num_config)
        };

        assert!(ret2 == 1, "Failed to choose a config");

        let usage = GRALLOC_USAGE_HW_FB | GRALLOC_USAGE_HW_RENDER | GRALLOC_USAGE_HW_COMPOSER;
        let native_window = GonkNativeWindow::new(alloc_dev, hwc_device, width, height, usage);
        let eglwindow = unsafe { egl::CreateWindowSurface(dpy, config, transmute(native_window), ptr::null()) };

        let ctx_attr =
            [egl::EGL_CONTEXT_CLIENT_VERSION, 2,
             egl::EGL_NONE, 0];

        let ctx = unsafe {
            egl::CreateContext(dpy, config, transmute(egl::EGL_NO_CONTEXT), transmute(ctx_attr.as_ptr()))
        };

        if ctx == unsafe { transmute(egl::EGL_NO_CONTEXT) } { panic!("Failed to create a context!") }

        unsafe {
            autosuspend_disable();
            ((*hwc_device).blank)(hwc_device, 0, 0);
        }

        let ret3 = egl::MakeCurrent(dpy, eglwindow, eglwindow, ctx);

        assert!(ret3 == 1, "Failed to make current!");

        unsafe {
            autosuspend_disable();
            ((*hwc_device).blank)(hwc_device, 0, 0);
        }

        unsafe {
            gl::ClearColor(1f32, 1f32, 1f32, 1f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        egl::SwapBuffers(dpy, eglwindow);

        let (tx, rx) = channel();

        // Create our window object.
        let window = Window {
            event_recv: rx,
            event_send: tx,
            width: width,
            height: height,
            native_window: native_window,
            dpy: dpy,
            ctx: ctx,
            surf: eglwindow,
        };

        Rc::new(window)
    }

    pub fn wait_events(&self) -> Vec<WindowEvent> {
        vec![self.event_recv.recv().unwrap()]
    }
}

impl Drop for Window {
    fn drop (&mut self) {
        unsafe {
            ((*self.native_window).window.common.decRef)(&mut (*self.native_window).window.common);
        }
    }
}

impl WindowMethods for Window {
    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, u32> {
        Size2D::typed(self.width as u32, self.height as u32)
    }

    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        Size2D::typed(self.width as f32, self.height as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self) {
        let _ = egl::SwapBuffers(self.dpy, self.surf);
    }

    fn set_page_title(&self, _: Option<String>) {
    }

    fn set_page_url(&self, _: Url) {
    }

    fn status(&self, _: Option<String>) {
    }

    fn load_start(&self, _: bool, _: bool) {
    }

    fn load_end(&self, _: bool, _: bool) {
    }

    fn load_error(&self, _: NetError, _: String) {
    }

    fn head_parsed(&self) {
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        ScaleFactor::new(1.0)
    }

    fn native_display(&self) -> NativeDisplay {
        NativeDisplay::new_with_display(self.dpy)
    }

    fn handle_key(&self, _: Key, _: KeyModifiers) {
    }

    fn create_compositor_channel(window: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy + Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box GonkCompositorProxy {
             sender: sender,
             event_sender: window.as_ref().unwrap().event_send.clone(),
         } as Box<CompositorProxy + Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn set_cursor(&self, _: Cursor) {
    }

    fn set_favicon(&self, _: Url) {
    }

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    fn supports_clipboard(&self) -> bool {
        true
    }
}

struct GonkCompositorProxy {
    sender: Sender<compositor_task::Msg>,
    event_sender: Sender<WindowEvent>,
}

impl CompositorProxy for GonkCompositorProxy {
    fn send(&self, msg: compositor_task::Msg) {
        // Send a message and kick the OS event loop awake.
        self.sender.send(msg).ok().unwrap();
        self.event_sender.send(WindowEvent::Idle).ok().unwrap();
    }
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy + Send> {
        box GonkCompositorProxy {
            sender: self.sender.clone(),
            event_sender: self.event_sender.clone(),
        } as Box<CompositorProxy + Send>
    }
}

