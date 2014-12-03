/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ConsoleBinding;
use dom::bindings::codegen::Bindings::ConsoleBinding::ConsoleMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use devtools_traits::{SendConsoleMessage, LogMessage};
use servo_util::str::DOMString;

#[dom_struct]
pub struct Console {
    reflector_: Reflector,
    global: GlobalField,
}

impl Console {
    fn new_inherited(global: &GlobalRef) -> Console {
        Console {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Console> {
        reflect_dom_object(box Console::new_inherited(global), global, ConsoleBinding::Wrap)
    }
}

impl<'a> ConsoleMethods for JSRef<'a, Console> {
    fn Log(self, message: DOMString) {
        println!("{:s}", message);
        propagate_console_msg(&self, message, LogMsg);
    }

    fn Debug(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Info(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Warn(self, message: DOMString) {
        println!("{:s}", message);
        propagate_console_msg(&self, message, WarnMsg);
    }

    fn Error(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Assert(self, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = match message {
                Some(ref message) => message.as_slice(),
                None => "no message",
            };
            println!("Assertion failed: {:s}", message);
        }
    }

}

impl Reflectable for Console {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

//TODO: Must be extended to contain all types of console message flavors:
// Error, Assert, Debug, Info
enum ConsoleMessageType {
    LogMsg,
    WarnMsg,
}

fn propagate_console_msg(console: &JSRef<Console>, message: DOMString, msg_type: ConsoleMessageType) {
    match msg_type {
        LogMsg => {
            let pipelineId = console.global.root().root_ref().as_window().page().id;
            console.global.root().root_ref().as_window().page().devtools_chan.as_ref().map(|chan| {
                chan.send(SendConsoleMessage(pipelineId, LogMessage(message.clone())));
            });
        }

        WarnMsg => {
            //TODO: to be implemented for warning messages
        }
    }
}
