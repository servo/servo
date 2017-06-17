/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use dom::bindings::inheritance::Castable;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::workerglobalscope::WorkerGlobalScope;
use std::io;

// https://developer.mozilla.org/en-US/docs/Web/API/Console
pub struct Console(());

impl Console {
    fn send_to_devtools(global: &GlobalScope, level: LogLevel, message: DOMString) {
        if let Some(chan) = global.devtools_chan() {
            let console_message = prepare_message(level, message);
            let worker_id = global.downcast::<WorkerGlobalScope>().map(|worker| {
                worker.get_worker_id()
            });
            let devtools_message = ScriptToDevtoolsControlMsg::ConsoleAPI(
                global.pipeline_id(),
                console_message,
                worker_id);
            chan.send(devtools_message).unwrap();
        }
    }
}

// In order to avoid interleaving the stdout output of the Console API methods
// with stderr that could be in use on other threads, we lock stderr until
// we're finished with stdout. Since the stderr lock is reentrant, there is
// no risk of deadlock if the callback ends up trying to write to stderr for
// any reason.
fn with_stderr_lock<F>(f: F) where F: FnOnce() {
    let stderr = io::stderr();
    let _handle = stderr.lock();
    f()
}

impl Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    pub fn Log(global: &GlobalScope, messages: Vec<DOMString>) {
        with_stderr_lock(move || {
            for message in messages {
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Log, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    pub fn Debug(global: &GlobalScope, messages: Vec<DOMString>) {
        with_stderr_lock(move || {
            for message in messages {
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Debug, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    pub fn Info(global: &GlobalScope, messages: Vec<DOMString>) {
        with_stderr_lock(move || {
            for message in messages {
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Info, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    pub fn Warn(global: &GlobalScope, messages: Vec<DOMString>) {
        with_stderr_lock(move || {
            for message in messages {
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Warn, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    pub fn Error(global: &GlobalScope, messages: Vec<DOMString>) {
        with_stderr_lock(move || {
            for message in messages {
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Error, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    pub fn Assert(global: &GlobalScope, condition: bool, message: Option<DOMString>) {
        with_stderr_lock(move || {
            if !condition {
                let message = message.unwrap_or_else(|| DOMString::from("no message"));
                println!("Assertion failed: {}", message);
                Self::send_to_devtools(global, LogLevel::Error, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/time
    pub fn Time(global: &GlobalScope, label: DOMString) {
        with_stderr_lock(move || {
            if let Ok(()) = global.time(label.clone()) {
                let message = DOMString::from(format!("{}: timer started", label));
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Log, message);
            }
        })
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/timeEnd
    pub fn TimeEnd(global: &GlobalScope, label: DOMString) {
        with_stderr_lock(move || {
            if let Ok(delta) = global.time_end(&label) {
                let message = DOMString::from(
                    format!("{}: {}ms", label, delta)
                );
                println!("{}", message);
                Self::send_to_devtools(global, LogLevel::Log, message);
            };
        })
    }
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
