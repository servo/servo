/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ConsoleBinding;
use dom::bindings::codegen::Bindings::ConsoleBinding::ConsoleMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use std::collections::HashMap;
use time::{Timespec, get_time};
use util::str::DOMString;

// https://developer.mozilla.org/en-US/docs/Web/API/Console
#[dom_struct]
pub struct Console {
    reflector_: Reflector,
    timers: DOMRefCell<HashMap<DOMString, u64>>,
}

impl Console {
    fn new_inherited() -> Console {
        Console {
            reflector_: Reflector::new(),
            timers: DOMRefCell::new(HashMap::new()),
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
}

impl ConsoleMethods for Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    fn Log(&self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            self.send_to_devtools(LogLevel::Log, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    fn Debug(&self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            self.send_to_devtools(LogLevel::Debug, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    fn Info(&self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            self.send_to_devtools(LogLevel::Info, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    fn Warn(&self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            self.send_to_devtools(LogLevel::Warn, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    fn Error(&self, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            self.send_to_devtools(LogLevel::Error, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    fn Assert(&self, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = message.unwrap_or_else(|| DOMString::from("no message"));
            println!("Assertion failed: {}", message);
            self.send_to_devtools(LogLevel::Error, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/time
    fn Time(&self, label: DOMString) {
        let mut timers = self.timers.borrow_mut();
        if timers.contains_key(&label) {
            // Timer already started
            return;
        }
        if timers.len() >= 10000 {
            // Too many timers on page
            return;
        }

        timers.insert(label.clone(), timestamp_in_ms(get_time()));
        let message = DOMString::from(format!("{}: timer started", label));
        println!("{}", message);
        self.send_to_devtools(LogLevel::Log, message);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/timeEnd
    fn TimeEnd(&self, label: DOMString) {
        let mut timers = self.timers.borrow_mut();
        if let Some(start) = timers.remove(&label) {
            let message = DOMString::from(
                format!("{}: {}ms", label, timestamp_in_ms(get_time()) - start)
            );
            println!("{}", message);
            self.send_to_devtools(LogLevel::Log, message);
        };
    }
}

fn timestamp_in_ms(time: Timespec) -> u64 {
    (time.sec * 1000 + (time.nsec / 1000000) as i64) as u64
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
