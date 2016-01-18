/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use dom::bindings::codegen::Bindings::ConsoleBinding;
use dom::bindings::codegen::Bindings::ConsoleBinding::ConsoleMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use util::str::DOMString;

// https://developer.mozilla.org/en-US/docs/Web/API/Console
#[dom_struct]
pub struct Console {
    reflector_: Reflector,
}

impl Console {
    fn new_inherited() -> Console {
        Console {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef) -> Root<Console> {
        reflect_dom_object(box Console::new_inherited(),
                           global,
                           ConsoleBinding::Wrap)
    }

    fn send_to_devtools(&self, level: LogLevel, message: DOMString) {
        let global = self.global();
        let global = global.r();
        if let Some(chan) = global.devtools_chan() {
            let console_message = prepare_message(level, message);
            let devtools_message = ScriptToDevtoolsControlMsg::ConsoleAPI(
                global.pipeline(),
                console_message,
                global.get_worker_id());
            chan.send(devtools_message).unwrap();
        }
    }

    fn send_messages(&self, log_level: LogLevel, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            self.send_to_devtools(log_level.clone(), message);
        }
    }
}

impl ConsoleMethods for Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    fn Log(&self, messages: Vec<DOMString>) {
        self.send_messages(LogLevel::Log, messages);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    fn Debug(&self, messages: Vec<DOMString>) {
        self.send_messages(LogLevel::Debug, messages);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    fn Info(&self, messages: Vec<DOMString>) {
        self.send_messages(LogLevel::Info, messages);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    fn Warn(&self, messages: Vec<DOMString>) {
        self.send_messages(LogLevel::Warn, messages);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    fn Error(&self, messages: Vec<DOMString>) {
        self.send_messages(LogLevel::Error, messages);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    fn Assert(&self, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = message.unwrap_or_else(|| DOMString::from("no message"));
            println!("Assertion failed: {}", message);
            self.send_to_devtools(LogLevel::Error, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/group
    fn Group(&self, messages: Vec<DOMString>) {
        self.send_messages(LogLevel::Group, messages);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/groupEnd
    fn GroupEnd(&self) {
        self.send_messages(LogLevel::GroupEnd, vec![]);
    }
}

fn prepare_message(logLevel: LogLevel, message: DOMString) -> ConsoleMessage {
    // TODO: Sending fake values for filename, lineNumber and columnNumber in LogMessage; adjust later
    ConsoleMessage {
        message: String::from(message),
        logLevel: logLevel,
        filename: "test".to_owned(),
        lineNumber: 1,
        columnNumber: 1,
    }
}
