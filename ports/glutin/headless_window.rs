/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use crate::window_trait::WindowPortsMethods;
use euclid::{TypedPoint2D, TypedScale, TypedSize2D};
use gleam::gl;
use servo::compositing::windowing::{AnimationState, WindowEvent};
use servo::compositing::windowing::{EmbedderCoordinates, WindowMethods};
use servo::servo_config::opts;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::{DeviceIntRect, FramebufferIntSize};
use std::cell::Cell;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct HeadlessContext {
    width: u32,
    height: u32,
    _context: osmesa_sys::OSMesaContext,
    _buffer: Vec<u32>,
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
struct HeadlessContext {
    width: u32,
    height: u32,
}

impl HeadlessContext {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn new(width: u32, height: u32) -> HeadlessContext {
        let mut attribs = Vec::new();

        attribs.push(osmesa_sys::OSMESA_PROFILE);
        attribs.push(osmesa_sys::OSMESA_CORE_PROFILE);
        attribs.push(osmesa_sys::OSMESA_CONTEXT_MAJOR_VERSION);
        attribs.push(3);
        attribs.push(osmesa_sys::OSMESA_CONTEXT_MINOR_VERSION);
        attribs.push(3);
        attribs.push(0);

        let context =
            unsafe { osmesa_sys::OSMesaCreateContextAttribs(attribs.as_ptr(), ptr::null_mut()) };

        assert!(!context.is_null());

        let mut buffer = vec![0; (width * height) as usize];

        unsafe {
            let ret = osmesa_sys::OSMesaMakeCurrent(
                context,
                buffer.as_mut_ptr() as *mut _,
                gl::UNSIGNED_BYTE,
                width as i32,
                height as i32,
            );
            assert_ne!(ret, 0);
        };

        HeadlessContext {
            width: width,
            height: height,
            _context: context,
            _buffer: buffer,
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn new(width: u32, height: u32) -> HeadlessContext {
        HeadlessContext {
            width: width,
            height: height,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn get_proc_address(s: &str) -> *const c_void {
        let c_str = CString::new(s).expect("Unable to create CString");
        unsafe { mem::transmute(osmesa_sys::OSMesaGetProcAddress(c_str.as_ptr())) }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn get_proc_address(_: &str) -> *const c_void {
        ptr::null() as *const _
    }
}

pub struct Window {
    context: HeadlessContext,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    gl: Rc<dyn gl::Gl>,
}

impl Window {
    pub fn new(size: TypedSize2D<u32, DeviceIndependentPixel>) -> Rc<dyn WindowPortsMethods> {
        let context = HeadlessContext::new(size.width, size.height);
        let gl = unsafe { gl::GlFns::load_with(|s| HeadlessContext::get_proc_address(s)) };

        // Print some information about the headless renderer that
        // can be useful in diagnosing CI failures on build machines.
        println!("{}", gl.get_string(gl::VENDOR));
        println!("{}", gl.get_string(gl::RENDERER));
        println!("{}", gl.get_string(gl::VERSION));

        let window = Window {
            context,
            gl,
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
        };

        Rc::new(window)
    }

    fn servo_hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        match opts::get().device_pixels_per_px {
            Some(device_pixels_per_px) => TypedScale::new(device_pixels_per_px),
            _ => TypedScale::new(1.0),
        }
    }
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<WindowEvent> {
        vec![]
    }

    fn has_events(&self) -> bool {
        false
    }

    fn id(&self) -> Option<glutin::WindowId> {
        None
    }

    fn page_height(&self) -> f32 {
        let dpr = self.servo_hidpi_factor();
        self.context.height as f32 * dpr.get()
    }

    fn set_fullscreen(&self, state: bool) {
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        return self.fullscreen.get();
    }

    fn is_animating(&self) -> bool {
        false
    }

    fn winit_event_to_servo_event(&self, _event: glutin::WindowEvent) {
        // Not expecting any winit events.
    }
}

impl WindowMethods for Window {
    fn gl(&self) -> Rc<dyn gl::Gl> {
        self.gl.clone()
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        let dpr = self.servo_hidpi_factor();
        let size =
            (TypedSize2D::new(self.context.width, self.context.height).to_f32() * dpr).to_i32();
        let viewport = DeviceIntRect::new(TypedPoint2D::zero(), size);
        let framebuffer = FramebufferIntSize::from_untyped(&size.to_untyped());
        EmbedderCoordinates {
            viewport,
            framebuffer,
            window: (size, TypedPoint2D::zero()),
            screen: size,
            screen_avail: size,
            hidpi_factor: dpr,
        }
    }

    fn present(&self) {}

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn prepare_for_composite(&self) -> bool {
        true
    }
}
