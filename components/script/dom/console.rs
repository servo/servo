/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryFrom;
use std::ptr::{self, NonNull};
use std::slice;

use devtools_traits::{
    ConsoleArgument, ConsoleArgumentObject, ConsoleArgumentPropertyValue, ConsoleLogLevel,
    ConsoleMessage, ConsoleMessageFields, ScriptToDevtoolsControlMsg, StackFrame, get_time_stamp,
};
use embedder_traits::EmbedderMsg;
use js::conversions::jsstr_to_string;
use js::jsapi::{self, ESClass, PropertyDescriptor};
use js::jsval::{Int32Value, UndefinedValue};
use js::rust::wrappers::{
    GetBuiltinClass, GetPropertyKeys, JS_GetOwnPropertyDescriptorById, JS_GetPropertyById,
    JS_IdToValue, JS_Stringify, JS_ValueToSource,
};
use js::rust::{
    CapturedJSStack, HandleObject, HandleValue, IdVector, ToString, describe_scripted_caller,
};
use script_bindings::conversions::get_dom_class;

use crate::dom::bindings::codegen::Bindings::ConsoleBinding::consoleMethods;
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
    #[expect(unsafe_code)]
    fn build_message(
        level: ConsoleLogLevel,
        arguments: Vec<ConsoleArgument>,
        stacktrace: Option<Vec<StackFrame>>,
    ) -> ConsoleMessage {
        let cx = GlobalScope::get_cx();
        let caller = unsafe { describe_scripted_caller(*cx) }.unwrap_or_default();

        ConsoleMessage {
            fields: ConsoleMessageFields {
                level,
                filename: caller.filename,
                line_number: caller.line,
                column_number: caller.col,
                time_stamp: get_time_stamp(),
            },
            arguments,
            stacktrace,
        }
    }

    /// Helper to send a message that only consists of a single string
    fn send_string_message(global: &GlobalScope, level: ConsoleLogLevel, message: String) {
        let prefix = global.current_group_label().unwrap_or_default();
        let formatted_message = format!("{prefix}{message}");

        Self::send_to_embedder(global, level.clone(), formatted_message);

        let console_message = Self::build_message(level, vec![message.into()], None);

        Self::send_to_devtools(global, console_message);
    }

    fn method(
        global: &GlobalScope,
        level: ConsoleLogLevel,
        messages: Vec<HandleValue>,
        include_stacktrace: IncludeStackTrace,
    ) {
        let cx = GlobalScope::get_cx();

        let arguments = messages
            .iter()
            .map(|msg| console_argument_from_handle_value(cx, *msg))
            .collect();
        let stacktrace = (include_stacktrace == IncludeStackTrace::Yes)
            .then_some(get_js_stack(*GlobalScope::get_cx()));
        let console_message = Self::build_message(level.clone(), arguments, stacktrace);

        Console::send_to_devtools(global, console_message);

        let prefix = global.current_group_label().unwrap_or_default();
        let msgs = stringify_handle_values(&messages);
        let formatted_message = format!("{prefix}{msgs}");

        Self::send_to_embedder(global, level, formatted_message);
    }

    fn send_to_devtools(global: &GlobalScope, message: ConsoleMessage) {
        if let Some(chan) = global.devtools_chan() {
            let worker_id = global
                .downcast::<WorkerGlobalScope>()
                .map(|worker| worker.worker_id());
            let devtools_message =
                ScriptToDevtoolsControlMsg::ConsoleAPI(global.pipeline_id(), message, worker_id);
            chan.send(devtools_message).unwrap();
        }
    }

    fn send_to_embedder(global: &GlobalScope, level: ConsoleLogLevel, message: String) {
        global.send_to_embedder(EmbedderMsg::ShowConsoleApiMessage(
            global.webview_id(),
            level,
            message,
        ));
    }

    // Directly logs a DOMString, without processing the message
    pub(crate) fn internal_warn(global: &GlobalScope, message: DOMString) {
        Console::send_string_message(global, ConsoleLogLevel::Warn, String::from(message.clone()));
    }
}

#[expect(unsafe_code)]
unsafe fn handle_value_to_string(cx: *mut jsapi::JSContext, value: HandleValue) -> DOMString {
    rooted!(in(cx) let mut js_string = std::ptr::null_mut::<jsapi::JSString>());
    match std::ptr::NonNull::new(unsafe { JS_ValueToSource(cx, value) }) {
        Some(js_str) => {
            js_string.set(js_str.as_ptr());
            DOMString::from_string(unsafe { jsstr_to_string(cx, js_str) })
        },
        None => "<error converting value to string>".into(),
    }
}

#[expect(unsafe_code)]
fn console_argument_from_handle_value(cx: JSContext, handle_value: HandleValue) -> ConsoleArgument {
    if handle_value.is_string() {
        let js_string = ptr::NonNull::new(handle_value.to_string()).unwrap();
        let dom_string = unsafe { jsstr_to_string(*cx, js_string) };
        return ConsoleArgument::String(dom_string);
    }

    if handle_value.is_int32() {
        let integer = handle_value.to_int32();
        return ConsoleArgument::Integer(integer);
    }

    if handle_value.is_number() {
        let number = handle_value.to_number();
        return ConsoleArgument::Number(number);
    }

    if handle_value.is_object() {
        if let Some(console_argument_object) = console_object_from_handle_value(cx, handle_value) {
            return ConsoleArgument::Object(console_argument_object);
        }
    }

    // FIXME: Handle more complex argument types here
    let stringified_value = stringify_handle_value(handle_value);
    ConsoleArgument::String(stringified_value.into())
}

#[expect(unsafe_code)]
fn console_object_from_handle_value(
    cx: JSContext,
    handle_value: HandleValue,
) -> Option<ConsoleArgumentObject> {
    rooted!(in(*cx) let object = handle_value.to_object());

    let mut own_properties = Vec::new();
    let mut ids = unsafe { IdVector::new(*cx) };
    if !unsafe {
        GetPropertyKeys(
            *cx,
            object.handle(),
            jsapi::JSITER_OWNONLY | jsapi::JSITER_SYMBOLS,
            ids.handle_mut(),
        )
    } {
        return None;
    }

    for id in ids.iter() {
        rooted!(in(*cx) let id = *id);
        rooted!(in(*cx) let mut descriptor = PropertyDescriptor::default());

        let mut is_none = false;
        if !unsafe {
            JS_GetOwnPropertyDescriptorById(
                *cx,
                object.handle(),
                id.handle(),
                descriptor.handle_mut(),
                &mut is_none,
            )
        } {
            return None;
        }

        rooted!(in(*cx) let mut property = UndefinedValue());
        if !unsafe { JS_GetPropertyById(*cx, object.handle(), id.handle(), property.handle_mut()) }
        {
            return None;
        }

        let key = if id.is_string() || id.is_symbol() || id.is_int() {
            rooted!(in(*cx) let mut key_value = UndefinedValue());
            let raw_id: jsapi::HandleId = id.handle().into();
            if !unsafe { JS_IdToValue(*cx, *raw_id.ptr, key_value.handle_mut()) } {
                continue;
            }
            rooted!(in(*cx) let js_string = key_value.to_string());
            let Some(js_string) = NonNull::new(js_string.get()) else {
                continue;
            };
            unsafe { jsstr_to_string(*cx, js_string) }
        } else {
            continue;
        };

        own_properties.push(ConsoleArgumentPropertyValue {
            key,
            configurable: descriptor.hasConfigurable_(),
            enumerable: descriptor.hasEnumerable_(),
            writable: descriptor.hasWritable_(),
            value: console_argument_from_handle_value(cx, property.handle()),
        });
    }

    Some(ConsoleArgumentObject {
        class: "Object".to_owned(),
        own_properties,
    })
}

#[expect(unsafe_code)]
fn stringify_handle_value(message: HandleValue) -> DOMString {
    let cx = GlobalScope::get_cx();
    unsafe {
        if message.is_string() {
            let jsstr = std::ptr::NonNull::new(message.to_string()).unwrap();
            return DOMString::from_string(jsstr_to_string(*cx, jsstr));
        }
        unsafe fn stringify_object_from_handle_value(
            cx: *mut jsapi::JSContext,
            value: HandleValue,
            parents: Vec<u64>,
        ) -> DOMString {
            rooted!(in(cx) let mut obj = value.to_object());
            let mut object_class = ESClass::Other;
            if !unsafe { GetBuiltinClass(cx, obj.handle(), &mut object_class as *mut _) } {
                return DOMString::from("/* invalid */");
            }
            let mut ids = unsafe { IdVector::new(cx) };
            if !unsafe {
                GetPropertyKeys(
                    cx,
                    obj.handle(),
                    jsapi::JSITER_OWNONLY | jsapi::JSITER_SYMBOLS,
                    ids.handle_mut(),
                )
            } {
                return DOMString::from("/* invalid */");
            }
            let truncate = ids.len() > MAX_LOG_CHILDREN;
            if object_class != ESClass::Array && object_class != ESClass::Object {
                if truncate {
                    return DOMString::from("…");
                } else {
                    return unsafe { handle_value_to_string(cx, value) };
                }
            }

            let mut explicit_keys = object_class == ESClass::Object;
            let mut props = Vec::with_capacity(ids.len());
            for id in ids.iter().take(MAX_LOG_CHILDREN) {
                rooted!(in(cx) let id = *id);
                rooted!(in(cx) let mut desc = PropertyDescriptor::default());

                let mut is_none = false;
                if !unsafe {
                    JS_GetOwnPropertyDescriptorById(
                        cx,
                        obj.handle(),
                        id.handle(),
                        desc.handle_mut(),
                        &mut is_none,
                    )
                } {
                    return DOMString::from("/* invalid */");
                }

                rooted!(in(cx) let mut property = UndefinedValue());
                if !unsafe {
                    JS_GetPropertyById(cx, obj.handle(), id.handle(), property.handle_mut())
                } {
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
                let value_string = stringify_inner(
                    unsafe { JSContext::from_ptr(cx) },
                    property.handle(),
                    parents.clone(),
                );
                if explicit_keys {
                    let key = if id.is_string() || id.is_symbol() || id.is_int() {
                        rooted!(in(cx) let mut key_value = UndefinedValue());
                        let raw_id: jsapi::HandleId = id.handle().into();
                        if !unsafe { JS_IdToValue(cx, *raw_id.ptr, key_value.handle_mut()) } {
                            return DOMString::from("/* invalid */");
                        }
                        unsafe { handle_value_to_string(cx, key_value.handle()) }
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

#[expect(unsafe_code)]
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
        jsstr_to_string(*cx, class_name)
            .replace("[object ", "")
            .replace("]", "")
    };
    let mut repr = format!("{} ", class_name);
    rooted!(in(*cx) let mut value = value.get());

    #[expect(unsafe_code)]
    unsafe extern "C" fn stringified(
        string: *const u16,
        len: u32,
        data: *mut std::ffi::c_void,
    ) -> bool {
        let s = data as *mut String;
        let string_chars = unsafe { slice::from_raw_parts(string, len as usize) };
        unsafe { (*s).push_str(&String::from_utf16_lossy(string_chars)) };
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

#[derive(Debug, Eq, PartialEq)]
enum IncludeStackTrace {
    Yes,
    No,
}

impl consoleMethods<crate::DomTypeHolder> for Console {
    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console/log>
    fn Log(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(
            global,
            ConsoleLogLevel::Log,
            messages,
            IncludeStackTrace::No,
        );
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console/clear>
    fn Clear(global: &GlobalScope) {
        if let Some(chan) = global.devtools_chan() {
            let worker_id = global
                .downcast::<WorkerGlobalScope>()
                .map(|worker| worker.worker_id());
            let devtools_message =
                ScriptToDevtoolsControlMsg::ClearConsole(global.pipeline_id(), worker_id);
            if let Err(error) = chan.send(devtools_message) {
                log::warn!("Error sending clear message to devtools: {error:?}");
            }
        }
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console>
    fn Debug(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(
            global,
            ConsoleLogLevel::Debug,
            messages,
            IncludeStackTrace::No,
        );
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console/info>
    fn Info(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(
            global,
            ConsoleLogLevel::Info,
            messages,
            IncludeStackTrace::No,
        );
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console/warn>
    fn Warn(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(
            global,
            ConsoleLogLevel::Warn,
            messages,
            IncludeStackTrace::No,
        );
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console/error>
    fn Error(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(
            global,
            ConsoleLogLevel::Error,
            messages,
            IncludeStackTrace::No,
        );
    }

    /// <https://console.spec.whatwg.org/#trace>
    fn Trace(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        Console::method(
            global,
            ConsoleLogLevel::Trace,
            messages,
            IncludeStackTrace::Yes,
        );
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/API/Console/assert>
    fn Assert(_cx: JSContext, global: &GlobalScope, condition: bool, messages: Vec<HandleValue>) {
        if !condition {
            let message = format!("Assertion failed: {}", stringify_handle_values(&messages));

            Console::send_string_message(global, ConsoleLogLevel::Log, message.clone());
        }
    }

    /// <https://console.spec.whatwg.org/#time>
    fn Time(global: &GlobalScope, label: DOMString) {
        if let Ok(()) = global.time(label.clone()) {
            let message = format!("{label}: timer started");
            Console::send_string_message(global, ConsoleLogLevel::Log, message.clone());
        }
    }

    /// <https://console.spec.whatwg.org/#timelog>
    fn TimeLog(_cx: JSContext, global: &GlobalScope, label: DOMString, data: Vec<HandleValue>) {
        if let Ok(delta) = global.time_log(&label) {
            let message = format!("{label}: {delta}ms {}", stringify_handle_values(&data));

            Console::send_string_message(global, ConsoleLogLevel::Log, message.clone());
        }
    }

    /// <https://console.spec.whatwg.org/#timeend>
    fn TimeEnd(global: &GlobalScope, label: DOMString) {
        if let Ok(delta) = global.time_end(&label) {
            let message = format!("{label}: {delta}ms");

            Console::send_string_message(global, ConsoleLogLevel::Log, message.clone());
        }
    }

    /// <https://console.spec.whatwg.org/#group>
    fn Group(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        global.push_console_group(stringify_handle_values(&messages));
    }

    /// <https://console.spec.whatwg.org/#groupcollapsed>
    fn GroupCollapsed(_cx: JSContext, global: &GlobalScope, messages: Vec<HandleValue>) {
        global.push_console_group(stringify_handle_values(&messages));
    }

    /// <https://console.spec.whatwg.org/#groupend>
    fn GroupEnd(global: &GlobalScope) {
        global.pop_console_group();
    }

    /// <https://console.spec.whatwg.org/#count>
    fn Count(global: &GlobalScope, label: DOMString) {
        let count = global.increment_console_count(&label);
        let message = format!("{label}: {count}");

        Console::send_string_message(global, ConsoleLogLevel::Log, message.clone());
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

#[expect(unsafe_code)]
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
            unsafe { jsstr_to_string(cx, nonnull_result) }
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
            unsafe { jsstr_to_string(cx, nonnull_result) }
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
