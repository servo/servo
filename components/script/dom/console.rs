/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use js::rust::describe_scripted_caller;
use std::io;

// https://developer.mozilla.org/en-US/docs/Web/API/Console
pub struct Console(());

impl Console {
    #[allow(unsafe_code)]
    fn send_to_devtools(global: &GlobalScope, level: LogLevel, message: DOMString) {
        if let Some(chan) = global.devtools_chan() {
            let caller = unsafe { describe_scripted_caller(*global.get_cx()) }.unwrap_or_default();
            let console_message = ConsoleMessage {
                message: String::from(message),
                logLevel: level,
                filename: caller.filename,
                lineNumber: caller.line as usize,
                columnNumber: caller.col as usize,
            };
            let worker_id = global
                .downcast::<WorkerGlobalScope>()
                .map(|worker| worker.get_worker_id());
            let devtools_message = ScriptToDevtoolsControlMsg::ConsoleAPI(
                global.pipeline_id(),
                console_message,
                worker_id,
            );
            chan.send(devtools_message).unwrap();
        }
    }
}

// In order to avoid interleaving the stdout output of the Console API methods
// with stderr that could be in use on other threads, we lock stderr until
// we're finished with stdout. Since the stderr lock is reentrant, there is
// no risk of deadlock if the callback ends up trying to write to stderr for
// any reason.
fn with_stderr_lock<F>(f: F)
where
    F: FnOnce(),
{
    let stderr = io::stderr();
    let _handle = stderr.lock();
    f()
}

fn console_messages(global: &GlobalScope, messages: &[DOMString], level: LogLevel) {
    console_message(global, DOMString::from(messages.join(" ")), level)
}

fn console_message(global: &GlobalScope, message: DOMString, level: LogLevel) {
    with_stderr_lock(move || {
        let prefix = global.current_group_label().unwrap_or_default();
        let message = DOMString::from(format!("{}{}", prefix, message));
        println!("{}", message);
        Console::send_to_devtools(global, level, message);
    })
}

#[allow(non_snake_case)]
impl Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    pub fn Log(global: &GlobalScope, messages: Vec<DOMString>) {
        console_messages(global, &messages, LogLevel::Log)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/clear
    pub fn Clear(global: &GlobalScope) {
        let message: Vec<DOMString> = Vec::new();
        console_messages(global, &message, LogLevel::Clear)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    pub fn Debug(global: &GlobalScope, messages: Vec<DOMString>) {
        console_messages(global, &messages, LogLevel::Debug)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    pub fn Info(global: &GlobalScope, messages: Vec<DOMString>) {
        console_messages(global, &messages, LogLevel::Info)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    pub fn Warn(global: &GlobalScope, messages: Vec<DOMString>) {
        console_messages(global, &messages, LogLevel::Warn)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    pub fn Error(global: &GlobalScope, messages: Vec<DOMString>) {
        console_messages(global, &messages, LogLevel::Error)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    pub fn Assert(global: &GlobalScope, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = message.unwrap_or_else(|| DOMString::from("no message"));
            let message = DOMString::from(format!("Assertion failed: {}", message));
            console_message(global, message, LogLevel::Error)
        };
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/time
    pub fn Time(global: &GlobalScope, label: DOMString) {
        if let Ok(()) = global.time(label.clone()) {
            let message = DOMString::from(format!("{}: timer started", label));
            console_message(global, message, LogLevel::Log);
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/timeEnd
    pub fn TimeEnd(global: &GlobalScope, label: DOMString) {
        if let Ok(delta) = global.time_end(&label) {
            let message = DOMString::from(format!("{}: {}ms", label, delta));
            console_message(global, message, LogLevel::Log);
        }
    }

    // https://console.spec.whatwg.org/#group
    pub fn Group(global: &GlobalScope, messages: Vec<DOMString>) {
        global.push_console_group(DOMString::from(messages.join(" ")));
    }

    // https://console.spec.whatwg.org/#groupcollapsed
    pub fn GroupCollapsed(global: &GlobalScope, messages: Vec<DOMString>) {
        global.push_console_group(DOMString::from(messages.join(" ")));
    }

    // https://console.spec.whatwg.org/#groupend
    pub fn GroupEnd(global: &GlobalScope) {
        global.pop_console_group();
    }
}
