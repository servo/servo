/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.

use dom::bindings::js::{JS, JSRef, Root};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::workerglobalscope::WorkerGlobalScope;
use dom::window::Window;
use script_task::ScriptChan;

use servo_net::resource_task::ResourceTask;

use js::jsapi::JSContext;

use url::Url;

pub enum GlobalRef<'a> {
    Window(JSRef<'a, Window>),
    Worker(JSRef<'a, WorkerGlobalScope>),
}

pub enum GlobalRoot<'a, 'b> {
    WindowRoot(Root<'a, 'b, Window>),
    WorkerRoot(Root<'a, 'b, WorkerGlobalScope>),
}

#[deriving(Encodable)]
pub enum GlobalField {
    WindowField(JS<Window>),
    WorkerField(JS<WorkerGlobalScope>),
}

impl<'a> GlobalRef<'a> {
    pub fn get_cx(&self) -> *mut JSContext {
        match *self {
            Window(ref window) => window.get_cx(),
            Worker(ref worker) => worker.get_cx(),
        }
    }

    pub fn as_window<'b>(&'b self) -> &'b JSRef<'b, Window> {
        match *self {
            Window(ref window) => window,
            Worker(_) => fail!("expected a Window scope"),
        }
    }

    pub fn resource_task(&self) -> ResourceTask {
        match *self {
            Window(ref window) => window.page().resource_task.deref().clone(),
            Worker(ref worker) => worker.resource_task().clone(),
        }
    }

    pub fn get_url(&self) -> Url {
        match *self {
            Window(ref window) => window.get_url(),
            Worker(ref worker) => worker.get_url().clone(),
        }
    }

    pub fn script_chan<'b>(&'b self) -> &'b ScriptChan {
        match *self {
            Window(ref window) => &window.script_chan,
            Worker(ref worker) => worker.script_chan(),
        }
    }
}

impl<'a> Reflectable for GlobalRef<'a> {
    fn reflector<'b>(&'b self) -> &'b Reflector {
        match *self {
            Window(ref window) => window.reflector(),
            Worker(ref worker) => worker.reflector(),
        }
    }
}

impl<'a, 'b> GlobalRoot<'a, 'b> {
    pub fn root_ref<'c>(&'c self) -> GlobalRef<'c> {
        match *self {
            WindowRoot(ref window) => Window(window.root_ref()),
            WorkerRoot(ref worker) => Worker(worker.root_ref()),
        }
    }
}

impl GlobalField {
    pub fn from_rooted(global: &GlobalRef) -> GlobalField {
        match *global {
            Window(ref window) => WindowField(JS::from_rooted(window)),
            Worker(ref worker) => WorkerField(JS::from_rooted(worker)),
        }
    }

    pub fn root(&self) -> GlobalRoot {
        match *self {
            WindowField(ref window) => WindowRoot(window.root()),
            WorkerField(ref worker) => WorkerRoot(worker.root()),
        }
    }
}
