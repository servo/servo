/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use dom::bindings::codegen::Bindings::ConsoleBinding;
use dom::bindings::codegen::Bindings::ConsoleBinding::ConsoleMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::WindowHelpers;
use util::str::DOMString;

// https://developer.mozilla.org/en-US/docs/Web/API/Console
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

    pub fn new(global: GlobalRef) -> Root<Console> {
        reflect_dom_object(box Console::new_inherited(global), global, ConsoleBinding::Wrap)
    }
}

impl<'a> ConsoleMethods for &'a Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    fn Log(self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            propagate_console_msg(&self, prepare_message(LogLevel::Log, message));
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    fn Debug(self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            propagate_console_msg(&self, prepare_message(LogLevel::Debug, message));
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    fn Info(self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            propagate_console_msg(&self, prepare_message(LogLevel::Info, message));
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    fn Warn(self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            propagate_console_msg(&self, prepare_message(LogLevel::Warn, message));
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    fn Error(self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            propagate_console_msg(&self, prepare_message(LogLevel::Error, message));
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    fn Assert(self, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = match message {
                Some(ref message) => &**message,
                None => "no message",
            };
            println!("Assertion failed: {}", message);
            propagate_console_msg(&self, prepare_message(LogLevel::Error, message.to_owned()));
        }
    }
}

fn prepare_message(logLevel: LogLevel, message: String) -> ConsoleMessage {
    //TODO: Sending fake values for filename, lineNumber and columnNumber in LogMessage; adjust later
    ConsoleMessage {
        message: message,
        logLevel: logLevel,
        filename: "test".to_owned(),
        lineNumber: 1,
        columnNumber: 1
    }
}

fn propagate_console_msg(console: &&Console, console_message: ConsoleMessage) {
    let global = console.global.root();
    let pipelineId = global.r().pipeline();
    global.r().devtools_chan().as_ref().map(|chan| {
        chan.send(ScriptToDevtoolsControlMsg::ConsoleAPI(
            pipelineId, console_message.clone(), global.r().get_worker_id())).unwrap();
    });
}
