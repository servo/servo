/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.

use dom::bindings::js::{JS, JSRef, Root};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::window::Window;
use page::Page;
use script_task::ScriptChan;

use js::jsapi::JSContext;

use url::Url;

pub enum GlobalRef<'a> {
    Window(JSRef<'a, Window>),
}

pub enum GlobalRoot<'a, 'b> {
    WindowRoot(Root<'a, 'b, Window>),
}

#[deriving(Encodable)]
pub enum GlobalField {
    WindowField(JS<Window>),
}

impl<'a> GlobalRef<'a> {
    pub fn get_cx(&self) -> *mut JSContext {
        match *self {
            Window(ref window) => window.get_cx(),
        }
    }

    pub fn as_window<'b>(&'b self) -> &'b JSRef<'b, Window> {
        match *self {
            Window(ref window) => window,
        }
    }

    pub fn page<'b>(&'b self) -> &'b Page {
        self.as_window().page()
    }

    pub fn get_url(&self) -> Url {
        self.as_window().get_url()
    }

    pub fn script_chan<'b>(&'b self) -> &'b ScriptChan {
        &self.as_window().script_chan
    }
}

impl<'a> Reflectable for GlobalRef<'a> {
    fn reflector<'b>(&'b self) -> &'b Reflector {
        match *self {
            Window(ref window) => window.reflector(),
        }
    }
}

impl<'a, 'b> GlobalRoot<'a, 'b> {
    pub fn root_ref<'c>(&'c self) -> GlobalRef<'c> {
        match *self {
            WindowRoot(ref window) => Window(window.root_ref()),
        }
    }
}

impl GlobalField {
    pub fn from_rooted(global: &GlobalRef) -> GlobalField {
        match *global {
            Window(ref window) => WindowField(JS::from_rooted(window)),
        }
    }

    pub fn root(&self) -> GlobalRoot {
        match *self {
            WindowField(ref window) => WindowRoot(window.root()),
        }
    }
}
