/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use crate::window_trait::WindowPortsMethods;
use euclid::{default::Size2D as UntypedSize2D, Point2D, Rotation3D, Scale, Size2D, UnknownUnit};
use gleam::gl;
use glutin;
use servo::compositing::windowing::{AnimationState, WindowEvent};
use servo::compositing::windowing::{EmbedderCoordinates, WindowMethods};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::units::{DeviceIntRect, DeviceIntSize};
use servo_media::player::context as MediaPlayerCtxt;
use std::cell::Cell;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::cell::RefCell;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct HeadlessContext {
    width: u32,
    height: u32,
    context: osmesa_sys::OSMesaContext,
    buffer: RefCell<Vec<u32>>,
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
struct HeadlessContext {
    width: u32,
    height: u32,
}

impl HeadlessContext {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn new(width: u32, height: u32, share: Option<&HeadlessContext>) -> HeadlessContext {
        let mut attribs = Vec::new();

        attribs.push(osmesa_sys::OSMESA_PROFILE);
        attribs.push(osmesa_sys::OSMESA_CORE_PROFILE);
        attribs.push(osmesa_sys::OSMESA_CONTEXT_MAJOR_VERSION);
        attribs.push(3);
        attribs.push(osmesa_sys::OSMESA_CONTEXT_MINOR_VERSION);
        attribs.push(3);
        attribs.push(0);

        let share = share.map_or(ptr::null_mut(), |share| share.context as *mut _);

        let context = unsafe { osmesa_sys::OSMesaCreateContextAttribs(attribs.as_ptr(), share) };

        assert!(!context.is_null());

        HeadlessContext {
            width: width,
            height: height,
            context: context,
            buffer: RefCell::new(vec![0; (width * height) as usize]),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn new(width: u32, height: u32, _share: Option<&HeadlessContext>) -> HeadlessContext {
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
    device_pixels_per_px: Option<f32>,
}

impl Window {
    pub fn new(
        size: Size2D<u32, DeviceIndependentPixel>,
        device_pixels_per_px: Option<f32>,
    ) -> Rc<dyn WindowPortsMethods> {
        let context = HeadlessContext::new(size.width, size.height, None);
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
            device_pixels_per_px,
        };

        Rc::new(window)
    }

    fn servo_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        match self.device_pixels_per_px {
            Some(device_pixels_per_px) => Scale::new(device_pixels_per_px),
            _ => Scale::new(1.0),
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

    fn id(&self) -> glutin::WindowId {
        unsafe { glutin::WindowId::dummy() }
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
        self.animation_state.get() == AnimationState::Animating
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
        let size = (Size2D::new(self.context.width, self.context.height).to_f32() * dpr).to_i32();
        let viewport = DeviceIntRect::new(Point2D::zero(), size);
        let framebuffer = DeviceIntSize::from_untyped(size.to_untyped());
        EmbedderCoordinates {
            viewport,
            framebuffer,
            window: (size, Point2D::zero()),
            screen: size,
            screen_avail: size,
            hidpi_factor: dpr,
        }
    }

    fn present(&self) {}

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn make_gl_context_current(&self) {
        unsafe {
            let mut buffer = self.context.buffer.borrow_mut();
            let ret = osmesa_sys::OSMesaMakeCurrent(
                self.context.context,
                buffer.as_mut_ptr() as *mut _,
                gl::UNSIGNED_BYTE,
                self.context.width as i32,
                self.context.height as i32,
            );
            assert_ne!(ret, 0);
        };
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn make_gl_context_current(&self) {}

    fn get_gl_context(&self) -> MediaPlayerCtxt::GlContext {
        MediaPlayerCtxt::GlContext::Unknown
    }

    fn get_native_display(&self) -> MediaPlayerCtxt::NativeDisplay {
        MediaPlayerCtxt::NativeDisplay::Unknown
    }

    fn get_gl_api(&self) -> MediaPlayerCtxt::GlApi {
        MediaPlayerCtxt::GlApi::None
    }
}

impl webxr::glwindow::GlWindow for Window {
    fn make_current(&self) {}
    fn swap_buffers(&self) {}
    fn size(&self) -> UntypedSize2D<gl::GLsizei> {
        let dpr = self.servo_hidpi_factor().get();
        Size2D::new(
            (self.context.width as f32 * dpr) as gl::GLsizei,
            (self.context.height as f32 * dpr) as gl::GLsizei,
        )
    }
    fn new_window(&self) -> Result<Rc<dyn webxr::glwindow::GlWindow>, ()> {
        let width = self.context.width;
        let height = self.context.height;
        let share = Some(&self.context);
        let context = HeadlessContext::new(width, height, share);
        let gl = self.gl.clone();
        Ok(Rc::new(Window {
            context,
            gl,
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            device_pixels_per_px: self.device_pixels_per_px,
        }))
    }
    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit> {
        Rotation3D::identity()
    }
}
