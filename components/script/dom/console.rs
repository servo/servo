/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::conversions::{
    is_array_like, FromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::script_runtime::JSContext as SafeJSContext;
use devtools_traits::{ConsoleMessage, LogLevel, ScriptToDevtoolsControlMsg};
use js::jsapi::JSContext;
use js::rust::describe_scripted_caller;
use js::rust::HandleValue;
use std::cmp::max;
use std::io;
use std::option::Option;

// https://developer.mozilla.org/en-US/docs/Web/API/Console
pub struct Console(());

impl Console {
    #[allow(unsafe_code)]
    fn send_to_devtools(global: &GlobalScope, level: LogLevel, message: DOMString) {
        if let Some(chan) = global.devtools_chan() {
            let caller =
                unsafe { describe_scripted_caller(*GlobalScope::get_cx()) }.unwrap_or_default();
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

/// Generates matrix of table elements, and then converts matrix into a properly formatted DOMString
// TODO: Implement generate_table for objects and arrays of objects
#[allow(unsafe_code)]
fn generate_table(cx: *mut JSContext, tabular_data: &HandleValue) -> Option<DOMString> {
    let arr_like: bool = unsafe { is_array_like(cx, *tabular_data) };
    if !tabular_data.get().is_object() || !arr_like {
        return None;
    }

    let vec: Vec<DOMString> = unsafe {
        Vec::<DOMString>::from_jsval(cx, *tabular_data, StringificationBehavior::Empty)
            .expect("Unable to convert Array object to Vec<DOMString>")
            .get_success_value()
            .unwrap_or(&Vec::new())
            .to_vec()
    };

    let mut table_matrix: Vec<Vec<DOMString>> = Vec::new();
    // Only two columns because we don't expect objects as input yet
    table_matrix.push(vec!["(index)".into(), "Values".into()]);
    for (index, word) in vec.into_iter().enumerate() {
        table_matrix.push(vec![index.to_string().into(), word]);
    }

    let mut result: DOMString = DOMString::new();
    let mut max_lengths: Vec<usize> = vec![0; table_matrix[0].len()];
    for row in table_matrix.iter() {
        for (index, column_value) in row.iter().enumerate() {
            let column_str = column_value as &str;
            max_lengths[index] = max(column_str.chars().count(), max_lengths[index]);
        }
    }

    for row in table_matrix.iter() {
        for (index, column) in row.iter().enumerate() {
            let column_str = column as &str;
            let padding_width = max_lengths[index] - column_str.chars().count() + 1;
            result.push_str(column_str);
            result.push_str(&" ".repeat(padding_width));
        }

        result.push_str("\n");
    }

    Some(result)
}

#[allow(non_snake_case)]
impl Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    pub fn Log(global: &GlobalScope, messages: Vec<DOMString>) {
        console_messages(global, &messages, LogLevel::Log)
    }

    // https://console.spec.whatwg.org/#table
    pub fn Table(
        cx: SafeJSContext,
        global: &GlobalScope,
        tabular_data: HandleValue,
        properties: Option<Vec<DOMString>>,
    ) {
        let cx_ptr: *mut JSContext = *cx as *mut JSContext;

        let table: Option<DOMString> = generate_table(cx_ptr, &tabular_data);

        match table {
            Some(table_string) => console_messages(global, &[table_string], LogLevel::Log),
            None => console_messages(global, &[], LogLevel::Log),
        }
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
