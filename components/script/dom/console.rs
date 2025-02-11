/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryFrom;
use std::ptr::{self, NonNull};
use std::{io, slice};

use devtools_traits::{
    ConsoleMessage, ConsoleMessageArgument, ConsoleMessageBuilder, LogLevel,
    ScriptToDevtoolsControlMsg, StackFrame,
};
use js::jsapi::{self, ESClass, PropertyDescriptor};
use js::jsval::{Int32Value, UndefinedValue};
use js::rust::wrappers::{
    GetBuiltinClass, GetPropertyKeys, JS_GetOwnPropertyDescriptorById, JS_GetPropertyById,
    JS_IdToValue, JS_Stringify, JS_ValueToSource,
};
use js::rust::{
    describe_scripted_caller, CapturedJSStack, HandleObject, HandleValue, IdVector, ToString,
};
use script_bindings::conversions::get_dom_class;

use crate::dom::bindings::codegen::Bindings::ConsoleBinding::consoleMethods;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::script_runtime::JSContext;

/// The maximum object depth logged by console methods.
const MAX_LOG_DEPTH: usize = 10;
/// The maximum elements in an object logged by console methods.
const MAX_LOG_CHILDREN: usize = 15;

/// <https://developer.mozilla.org/en-US/docs/Web/API/Console>
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Console;

impl Console {
    #[allow(unsafe_code)]
    fn build_message(level: LogLevel) -> ConsoleMessageBuilder {
        let cx = GlobalScope::get_cx();
        let caller = unsafe { describe_scripted_caller(*cx) }.unwrap_or_default();

        ConsoleMessageBuilder::new(level, caller.filename, caller.line, caller.col)
    }

    /// Helper to send a message that only consists of a single string to the devtools
    fn send_string_message(global: &GlobalScope, level: LogLevel, message: String) {
        let mut builder = Self::build_message(level);
        builder.add_argument(message.into());
        let log_message = builder.finish();

        Self::send_to_devtools(global, log_message);
    }

    fn method(
        global: &GlobalScope,
        level: LogLevel,
        messages: Vec<HandleValue>,
        include_stacktrace: IncludeStackTrace,
    ) {
        let cx = GlobalScope::get_cx();

        let mut log: ConsoleMessageBuilder = Console::build_message(level);
        for message in &messages {
            log.add_argument(console_argument_from_handle_value(cx, *message));
        }

        if include_stacktrace == IncludeStackTrace::Yes {
            log.attach_stack_trace(get_js_stack(*GlobalScope::get_cx()));
        }

        Console::send_to_devtools(global, log.finish());

        // Also log messages to stdout
        console_messages(global, messages)
    }

    fn send_to_devtools(global: &GlobalScope, message: ConsoleMessage) {
        if let Some(chan) = global.devtools_chan() {
            let worker_id = global
                .downcast::<WorkerGlobalScope>()
                .map(|worker| worker.get_worker_id());
            let devtools_message =
                ScriptToDevtoolsControlMsg::ConsoleAPI(global.pipeline_id(), message, worker_id);
            chan.send(devtools_message).unwrap();
        }
    }

    // Directly logs a DOMString, without processing the message
    pub(crate) fn internal_warn(global: &GlobalScope, message: DOMString) {
        Console::send_string_message(global, LogLevel::Warn, String::from(message.clone()));
        console_message(global, message);
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

#[allow(unsafe_code)]
unsafe fn handle_value_to_string(cx: *mut jsapi::JSContext, value: HandleValue) -> DOMString {
    rooted!(in(cx) let mut js_string = std::ptr::null_mut::<jsapi::JSString>());
    match std::ptr::NonNull::new(JS_ValueToSource(cx, value)) {
        Some(js_str) => {
            js_string.set(js_str.as_ptr());
            jsstring_to_str(cx, js_str)
        },
        None => "<error converting value to string>".into(),
    }
}

#[allow(unsafe_code)]
fn console_argument_from_handle_value(
    cx: JSContext,
    handle_value: HandleValue,
) -> ConsoleMessageArgument {
    if handle_value.is_string() {
        let js_string = ptr::NonNull::new(handle_value.to_string()).unwrap();
        let dom_string = unsafe { jsstring_to_str(*cx, js_string) };
        return ConsoleMessageArgument::String(dom_string.into());
    }

    if handle_value.is_int32() {
        let integer = handle_value.to_int32();
        return ConsoleMessageArgument::Integer(integer);
    }

    if handle_value.is_number() {
        let number = handle_value.to_number();
        return ConsoleMessageArgument::Number(number);
    }

    // FIXME: Handle more complex argument types here
    let stringified_value = stringify_handle_value(handle_value);
    ConsoleMessageArgument::String(stringified_value.into())
}

#[allow(unsafe_code)]
fn stringify_handle_value(message: HandleValue) -> DOMString {
    let cx = GlobalScope::get_cx();
    unsafe {
        if message.is_string() {
            return jsstring_to_str(*cx, std::ptr::NonNull::new(message.to_string()).unwrap());
        }
        unsafe fn stringify_object_from_handle_value(
            cx: *mut jsapi::JSContext,
            value: HandleValue,
            parents: Vec<u64>,
        ) -> DOMString {
            rooted!(in(cx) let mut obj = value.to_object());
            let mut object_class = ESClass::Other;
            if !GetBuiltinClass(cx, obj.handle(), &mut object_class as *mut _) {
                return DOMString::from("/* invalid */");
            }
            let mut ids = IdVector::new(cx);
            if !GetPropertyKeys(
                cx,
                obj.handle(),
                jsapi::JSITER_OWNONLY | jsapi::JSITER_SYMBOLS,
                ids.handle_mut(),
            ) {
                return DOMString::from("/* invalid */");
            }
            let truncate = ids.len() > MAX_LOG_CHILDREN;
            if object_class != ESClass::Array && object_class != ESClass::Object {
                if truncate {
                    return DOMString::from("…");
                } else {
                    return handle_value_to_string(cx, value);
                }
            }

            let mut explicit_keys = object_class == ESClass::Object;
            let mut props = Vec::with_capacity(ids.len());
            for id in ids.iter().take(MAX_LOG_CHILDREN) {
                rooted!(in(cx) let id = *id);
                rooted!(in(cx) let mut desc = PropertyDescriptor::default());

                let mut is_none = false;
                if !JS_GetOwnPropertyDescriptorById(
                    cx,
                    obj.handle(),
                    id.handle(),
                    desc.handle_mut(),
                    &mut is_none,
                ) {
                    return DOMString::from("/* invalid */");
                }

                rooted!(in(cx) let mut property = UndefinedValue());
                if !JS_GetPropertyById(cx, obj.handle(), id.handle(), property.handle_mut()) {
                    return DOMString::from("/* invalid */");
                }

                if !explicit_keys {
                    if id.is_int() {
                        if let Ok(id_int) = usize::try_from(id.to_int()) {
                            explicit_keys = props.len() != id_int;
                        } else {
                            explicit_keys = false;
                        }
                    } else {
                        explicit_keys = false;
                    }
                }
                let value_string =
                    stringify_inner(JSContext::from_ptr(cx), property.handle(), parents.clone());
                if explicit_keys {
                    let key = if id.is_string() || id.is_symbol() || id.is_int() {
                        rooted!(in(cx) let mut key_value = UndefinedValue());
                        let raw_id: jsapi::HandleId = id.handle().into();
                        if !JS_IdToValue(cx, *raw_id.ptr, key_value.handle_mut()) {
                            return DOMString::from("/* invalid */");
                        }
                        handle_value_to_string(cx, key_value.handle())
                    } else {
                        return DOMString::from("/* invalid */");
                    };
                    props.push(format!("{}: {}", key, value_string,));
                } else {
                    props.push(value_string.to_string());
                }
            }
            if truncate {
                props.push("…".to_string());
            }
            if object_class == ESClass::Array {
                DOMString::from(format!("[{}]", itertools::join(props, ", ")))
            } else {
                DOMString::from(format!("{{{}}}", itertools::join(props, ", ")))
            }
        }
        fn stringify_inner(cx: JSContext, value: HandleValue, mut parents: Vec<u64>) -> DOMString {
            if parents.len() >= MAX_LOG_DEPTH {
                return DOMString::from("...");
            }
            let value_bits = value.asBits_;
            if parents.contains(&value_bits) {
                return DOMString::from("[circular]");
            }
            if value.is_undefined() {
                // This produces a better value than "(void 0)" from JS_ValueToSource.
                return DOMString::from("undefined");
            } else if !value.is_object() {
                return unsafe { handle_value_to_string(*cx, value) };
            }
            parents.push(value_bits);

            if value.is_object() {
                if let Some(repr) = maybe_stringify_dom_object(cx, value) {
                    return repr;
                }
            }
            unsafe { stringify_object_from_handle_value(*cx, value, parents) }
        }
        stringify_inner(cx, message, Vec::new())
    }
}

#[allow(unsafe_code)]
fn maybe_stringify_dom_object(cx: JSContext, value: HandleValue) -> Option<DOMString> {
    // The standard object serialization is not effective for DOM objects,
    // since their properties generally live on the prototype object.
    // Instead, fall back to the output of JSON.stringify combined
    // with the class name extracted from the output of toString().
    rooted!(in(*cx) let obj = value.to_object());
    let is_dom_class = unsafe { get_dom_class(obj.get()).is_ok() };
    if !is_dom_class {
        return None;
    }
    rooted!(in(*cx) let class_name = unsafe { ToString(*cx, value) });
    let Some(class_name) = NonNull::new(class_name.get()) else {
        return Some("<error converting DOM object to string>".into());
    };
    let class_name = unsafe {
        jsstring_to_str(*cx, class_name)
            .replace("[object ", "")
            .replace("]", "")
    };
    let mut repr = format!("{} ", class_name);
    rooted!(in(*cx) let mut value = value.get());

    #[allow(unsafe_code)]
    unsafe extern "C" fn stringified(
        string: *const u16,
        len: u32,
        data: *mut std::ffi::c_void,
    ) -> bool {
        let s = data as *mut String;
        let string_chars = slice::from_raw_parts(string, len as usize);
        (*s).push_str(&String::from_utf16_lossy(string_chars));
        true
    }

    rooted!(in(*cx) let space = Int32Value(2));
    let stringify_result = unsafe {
        JS_Stringify(
            *cx,
            value.handle_mut(),
            HandleObject::null(),
            space.handle(),
            Some(stringified),
            &mut repr as *mut String as *mut _,
        )
    };
    if !stringify_result {
        return Some("<error converting DOM object to string>".into());
    }
    Some(repr.into())
}

fn stringify_handle_values(messages: &[HandleValue]) -> DOMString {
    DOMString::from(itertools::join(
        messages.iter().copied().map(stringify_handle_value),
        " ",
    ))
}

fn console_messages(global: &GlobalScope, messages: Vec<HandleValue>) {
    let message = stringify_handle_values(&messages);
    console_message(global, message)
}

fn console_message(global: &GlobalScope, message: DOMString) {
    with_stderr_lock(move || {
        let prefix = global.current_group_label().unwrap_or_default();
        let message = format!("{}{}", prefix, message);
        println!("{}", message);
    })
}

#[derive(Debug, Eq, PartialEq)]
enum IncludeStackTrace {
    Yes,
    No,
}

impl consoleMethods<crate::DomTypeHolder> for Console {
    // https://developer.mozilla.org/en-US/docs/Web/API/Console/log
    fn Log(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(global, LogLevel::Log, messages, IncludeStackTrace::No);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/clear
    fn Clear(global: &GlobalScope) {
        let message = Console::build_message(LogLevel::Clear).finish();
        Console::send_to_devtools(global, message);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console
    fn Debug(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(global, LogLevel::Debug, messages, IncludeStackTrace::No);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/info
    fn Info(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(global, LogLevel::Info, messages, IncludeStackTrace::No);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/warn
    fn Warn(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(global, LogLevel::Warn, messages, IncludeStackTrace::No);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/error
    fn Error(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(global, LogLevel::Error, messages, IncludeStackTrace::No);
    }

    /// <https://console.spec.whatwg.org/#trace>
    fn Trace(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(global, LogLevel::Trace, messages, IncludeStackTrace::Yes);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/Console/assert
    fn Assert(_cx: JSContext, global: &GlobalScope, condition: bool, messages: Vec<HandleValue>) {
        if !condition {
            let message = format!("Assertion failed: {}", stringify_handle_values(&messages));

            Console::send_string_message(global, LogLevel::Log, message.clone());
            console_message(global, DOMString::from(message));
        }
    }

    // https://console.spec.whatwg.org/#time
    fn Time(global: &GlobalScope, label: DOMString) {
        if let Ok(()) = global.time(label.clone()) {
            let message = format!("{label}: timer started");
            Console::send_string_message(global, LogLevel::Log, message.clone());
            console_message(global, DOMString::from(message));
        }
    }

    // https://console.spec.whatwg.org/#timelog
    fn TimeLog(_cx: JSContext, global: &GlobalScope, label: DOMString, data: Vec<HandleValue>) {
        if let Ok(delta) = global.time_log(&label) {
            let message = format!("{label}: {delta}ms {}", stringify_handle_values(&data));

            Console::send_string_message(global, LogLevel::Log, message.clone());
            console_message(global, DOMString::from(message));
        }
    }

    // https://console.spec.whatwg.org/#timeend
    fn TimeEnd(global: &GlobalScope, label: DOMString) {
        if let Ok(delta) = global.time_end(&label) {
            let message = format!("{label}: {delta}ms");

            Console::send_string_message(global, LogLevel::Log, message.clone());
            console_message(global, DOMString::from(message));
        }
    }

    // https://console.spec.whatwg.org/#group
    fn Group(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        global.push_console_group(stringify_handle_values(&messages));
    }

    // https://console.spec.whatwg.org/#groupcollapsed
    fn GroupCollapsed(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        global.push_console_group(stringify_handle_values(&messages));
    }

    // https://console.spec.whatwg.org/#groupend
    fn GroupEnd(global: &GlobalScope) {
        global.pop_console_group();
    }

    /// <https://console.spec.whatwg.org/#count>
    fn Count(global: &GlobalScope, label: DOMString) {
        let count = global.increment_console_count(&label);
        let message = format!("{label}: {count}");

        Console::send_string_message(global, LogLevel::Log, message.clone());
        console_message(global, DOMString::from(message));
    }

    /// <https://console.spec.whatwg.org/#countreset>
    fn CountReset(global: &GlobalScope, label: DOMString) {
        if global.reset_console_count(&label).is_err() {
            Self::internal_warn(
                global,
                DOMString::from(format!("Counter “{label}” doesn’t exist.")),
            )
        }
    }
}

#[allow(unsafe_code)]
fn get_js_stack(cx: *mut jsapi::JSContext) -> Vec<StackFrame> {
    const MAX_FRAME_COUNT: u32 = 128;

    let mut frames = vec![];
    rooted!(in(cx) let mut handle =  ptr::null_mut());
    let captured_js_stack = unsafe { CapturedJSStack::new(cx, handle, Some(MAX_FRAME_COUNT)) };
    let Some(captured_js_stack) = captured_js_stack else {
        return frames;
    };

    captured_js_stack.for_each_stack_frame(|frame| {
        rooted!(in(cx) let mut result: *mut jsapi::JSString = ptr::null_mut());

        // Get function name
        unsafe {
            jsapi::GetSavedFrameFunctionDisplayName(
                cx,
                ptr::null_mut(),
                frame.into(),
                result.handle_mut().into(),
                jsapi::SavedFrameSelfHosted::Include,
            );
        }
        let function_name = if let Some(nonnull_result) = ptr::NonNull::new(*result) {
            unsafe { jsstring_to_str(cx, nonnull_result) }.into()
        } else {
            "<anonymous>".into()
        };

        // Get source file name
        result.set(ptr::null_mut());
        unsafe {
            jsapi::GetSavedFrameSource(
                cx,
                ptr::null_mut(),
                frame.into(),
                result.handle_mut().into(),
                jsapi::SavedFrameSelfHosted::Include,
            );
        }
        let filename = if let Some(nonnull_result) = ptr::NonNull::new(*result) {
            unsafe { jsstring_to_str(cx, nonnull_result) }.into()
        } else {
            "<anonymous>".into()
        };

        // get line/column number
        let mut line_number = 0;
        unsafe {
            jsapi::GetSavedFrameLine(
                cx,
                ptr::null_mut(),
                frame.into(),
                &mut line_number,
                jsapi::SavedFrameSelfHosted::Include,
            );
        }

        let mut column_number = jsapi::JS::TaggedColumnNumberOneOrigin { value_: 0 };
        unsafe {
            jsapi::GetSavedFrameColumn(
                cx,
                ptr::null_mut(),
                frame.into(),
                &mut column_number,
                jsapi::SavedFrameSelfHosted::Include,
            );
        }
        let frame = StackFrame {
            filename,
            function_name,
            line_number,
            column_number: column_number.value_,
        };

        frames.push(frame);
    });

    frames
}
