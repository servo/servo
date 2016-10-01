/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::global::GlobalRef;
use dom::bindings::str::DOMString;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use time::{Timespec, get_time};

// https://developer.mozilla.org/en-US/docs/Web/API/Console
pub struct Console(());

impl Console {
    fn send_to_devtools(global: GlobalRef, level: LogLevel, message: DOMString) {
        if let Some(chan) = global.devtools_chan() {
            let console_message = prepare_message(level, message);
            let worker_id = if let GlobalRef::Worker(worker) = global {
                Some(worker.get_worker_id())
            } else {
                None
            };
            let devtools_message = ScriptToDevtoolsControlMsg::ConsoleAPI(
                global.pipeline_id(),
                console_message,
                worker_id);
            chan.send(devtools_message).unwrap();
        }
    }
}

impl Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    pub fn Log(global: GlobalRef, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Log, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    pub fn Debug(global: GlobalRef, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Debug, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    pub fn Info(global: GlobalRef, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Info, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    pub fn Warn(global: GlobalRef, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Warn, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    pub fn Error(global: GlobalRef, messages: Vec<DOMString>) {
        for message in messages {
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Error, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    pub fn Assert(global: GlobalRef, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = message.unwrap_or_else(|| DOMString::from("no message"));
            println!("Assertion failed: {}", message);
            Self::send_to_devtools(global, LogLevel::Error, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/time
    pub fn Time(global: GlobalRef, label: DOMString) {
        if let Ok(()) = global.console_timers().time(label.clone()) {
            let message = DOMString::from(format!("{}: timer started", label));
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Log, message);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/timeEnd
    pub fn TimeEnd(global: GlobalRef, label: DOMString) {
        if let Ok(delta) = global.console_timers().time_end(&label) {
            let message = DOMString::from(
                format!("{}: {}ms", label, delta)
            );
            println!("{}", message);
            Self::send_to_devtools(global, LogLevel::Log, message);
        };
    }
}

fn timestamp_in_ms(time: Timespec) -> u64 {
    (time.sec * 1000 + (time.nsec / 1000000) as i64) as u64
}

fn prepare_message(log_level: LogLevel, message: DOMString) -> ConsoleMessage {
    // TODO: Sending fake values for filename, lineNumber and columnNumber in LogMessage; adjust later
    ConsoleMessage {
        message: String::from(message),
        logLevel: log_level,
        filename: "test".to_owned(),
        lineNumber: 1,
        columnNumber: 1,
    }
}

#[derive(HeapSizeOf, JSTraceable)]
pub struct TimerSet(DOMRefCell<HashMap<DOMString, u64>>);

impl TimerSet {
    pub fn new() -> Self {
        TimerSet(DOMRefCell::new(Default::default()))
    }

    fn time(&self, label: DOMString) -> Result<(), ()> {
        let mut timers = self.0.borrow_mut();
        if timers.len() >= 10000 {
            return Err(());
        }
        match timers.entry(label) {
            Entry::Vacant(entry) => {
                entry.insert(timestamp_in_ms(get_time()));
                Ok(())
            },
            Entry::Occupied(_) => Err(()),
        }
    }

    fn time_end(&self, label: &str) -> Result<u64, ()> {
        self.0.borrow_mut().remove(label).ok_or(()).map(|start| {
            timestamp_in_ms(get_time()) - start
        })
    }
}
