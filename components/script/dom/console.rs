/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ConsoleBinding;
use dom::bindings::codegen::Bindings::ConsoleBinding::ConsoleMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use devtools_traits::{DevtoolsControlMsg, ConsoleMessage};
use util::str::DOMString;

#[dom_struct]
pub struct Console {
    reflector_: Reflector,
    global: GlobalField,
}

impl Console {
    fn new_inherited(global: GlobalRef) -> Console {
        Console {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
        }
    }

    pub fn new(global: GlobalRef) -> Temporary<Console> {
        reflect_dom_object(box Console::new_inherited(global), global, ConsoleBinding::Wrap)
    }
}

impl<'a> ConsoleMethods for JSRef<'a, Console> {
    fn Log(self, message: DOMString) {
        println!("{}", message);
        propagate_console_msg(&self, ConsoleMessage::LogMessage(message));
    }

    fn Debug(self, message: DOMString) {
        println!("{}", message);
    }

    fn Info(self, message: DOMString) {
        println!("{}", message);
    }

    fn Warn(self, message: DOMString) {
        println!("{}", message);
    }

    fn Error(self, message: DOMString) {
        println!("{}", message);
    }

    fn Assert(self, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = match message {
                Some(ref message) => message.as_slice(),
                None => "no message",
            };
            println!("Assertion failed: {}", message);
        }
    }
}

fn propagate_console_msg(console: &JSRef<Console>, console_message: ConsoleMessage) {
    let global = console.global.root();
    match global.r() {
        GlobalRef::Window(window_ref) => {
            let pipelineId = window_ref.page().id;
            console.global.root().r().as_window().page().devtools_chan.as_ref().map(|chan| {
                chan.send(DevtoolsControlMsg::SendConsoleMessage(
                    pipelineId, console_message.clone())).unwrap();
            });
        },

        GlobalRef::Worker(_) => {
            // TODO: support worker console logs
        }
    }
}
