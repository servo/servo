/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use glutin::os::ContextTraitExt;
use glutin::{ContextBuilder, CreationError, EventsLoop, NotCurrent, PossiblyCurrent, WindowedContext, WindowBuilder};
use servo_media::player::context::GlContext as RawContext;
use std::os::raw;

pub enum GlContext {
    Current(WindowedContext<PossiblyCurrent>),
    NotCurrent(WindowedContext<NotCurrent>),
    // Used a temporary value as we switch from Current to NotCurrent.
    None,
}

impl GlContext {
    pub fn window(&self) -> &glutin::Window {
        match self {
            GlContext::Current(c) => c.window(),
            GlContext::NotCurrent(c) => c.window(),
            GlContext::None => unreachable!(),
        }
    }
    pub fn resize(&mut self, size: glutin::dpi::PhysicalSize) {
        if let GlContext::NotCurrent(_) = self {
            self.make_current();
        }
        match self {
            GlContext::Current(c) => c.resize(size),
            _ => unreachable!(),
        }
    }
    pub fn make_current(&mut self) {
        *self = match std::mem::replace(self, GlContext::None) {
            GlContext::Current(c) => {
                warn!("Making an already current context current");
                GlContext::Current(c)
            },
            GlContext::NotCurrent(c) => {
                let c = unsafe {
                    c.make_current().expect("Couldn't make window current")
                };
                GlContext::Current(c)
            }
            GlContext::None => unreachable!(),
        }
    }
    pub fn make_not_current(&mut self) {
        *self = match std::mem::replace(self, GlContext::None) {
            GlContext::Current(c) => {
                let c = unsafe {
                    c.make_not_current().expect("Couldn't make window not current")
                };
                GlContext::NotCurrent(c)
            },
            GlContext::NotCurrent(c) => {
                warn!("Making an already not current context not current");
                GlContext::NotCurrent(c)
            }
            GlContext::None => unreachable!(),
        }
    }
    pub fn swap_buffers(&self) {
        match self {
            GlContext::Current(c) => {
                if let Err(err) = c.swap_buffers() {
                    warn!("Failed to swap window buffers ({}).", err);
                }
            },
            GlContext::NotCurrent(_) => {
                error!("Context is not current. Forgot to call prepare_for_composite?");
            },
            GlContext::None => unreachable!(),
        };
    }
    #[allow(unreachable_code, unused_variables)]
    pub fn raw_context(&self) -> RawContext {
        match self {
            GlContext::Current(c) => {
                #[cfg(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd",
                ))]
                {
                    use glutin::os::unix::RawHandle;

                    let raw_handle = unsafe { c.raw_handle() };
                    return match raw_handle {
                        RawHandle::Egl(handle) => RawContext::Egl(handle as usize),
                        RawHandle::Glx(handle) => RawContext::Glx(handle as usize),
                    };
                }

                #[cfg(target_os = "windows")]
                {
                    use glutin::os::windows::RawHandle;

                    let raw_handle = unsafe { c.raw_handle() };
                    return match raw_handle {
                        RawHandle::Egl(handle) => RawContext::Egl(handle as usize),
                        // @TODO(victor): RawContext::Wgl in servo-media
                        RawHandle::Wgl(_) => unimplemented!(),
                    }
                }

                #[cfg(target_os = "android")]
                {
                    let raw_handle = unsafe { c.raw_handle() };
                    return RawContext::Egl(raw_handle as usize);
                }

                #[cfg(target_os = "macos")]
                return unimplemented!(); // @TODO(victor): RawContext::Cocoa in servo-media

                #[cfg(not(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd",
                    target_os = "windows",
                    target_os = "android",
                    target_os = "macos",
                )))]
                unimplemented!()
            }
            GlContext::NotCurrent(_) => {
                error!("Context is not current.");
                RawContext::Unknown
            }
            GlContext::None => unreachable!(),
        }
    }

    pub fn new_window<T>(
        &self,
        cb: ContextBuilder<T>,
        wb: WindowBuilder,
        el: &EventsLoop
    ) -> Result<WindowedContext<NotCurrent>, CreationError>
    where
        T: glutin::ContextCurrentState,
    {
        match self {
            GlContext::Current(ref c) => {
                cb.with_shared_lists(c).build_windowed(wb, el)
            },
            GlContext::NotCurrent(ref c) => {
                cb.with_shared_lists(c).build_windowed(wb, el)
            },
            GlContext::None => {
                cb.build_windowed(wb, el)
            },
        }
    }

    #[allow(dead_code)]
    pub fn egl_display(&self) -> Option<*const raw::c_void> {
        match self {
            GlContext::Current(c) => unsafe { c.get_egl_display() },
            GlContext::NotCurrent(_) => {
                error!("Context is not current.");
                None
            },
            GlContext::None => unreachable!(),
        }
    }

    pub fn get_api(&self) -> glutin::Api {
        match self {
            GlContext::Current(c) => c.get_api(),
            GlContext::NotCurrent(c) => c.get_api(),
            GlContext::None => unreachable!(),
        }
    }
}
