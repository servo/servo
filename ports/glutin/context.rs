/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use glutin::{WindowedContext, NotCurrent, PossiblyCurrent};

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
}
