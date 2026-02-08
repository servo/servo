/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::ffi::CStr;
use std::ptr::NonNull;
use std::rc::Rc;

use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use js::jsapi::{
    Compile1, ExceptionStackBehavior, JS_ClearPendingException, JSScript, SetScriptPrivate,
};
use js::jsval::{PrivateValue, UndefinedValue};
use js::panic::maybe_resume_unwind;
use js::rust::wrappers::{
    JS_ExecuteScript, JS_GetPendingException, JS_GetScriptPrivate, JS_SetPendingException,
};
use js::rust::{CompileOptionsWrapper, MutableHandleValue, transform_str_to_source_text};
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::script_module::{ModuleScript, ModuleSource, ModuleTree, RethrowError, ScriptFetchOptions};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::unminify::unminify_js;

/// <https://html.spec.whatwg.org/multipage/#classic-script>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ClassicScript {
    /// On script parsing success this will be <https://html.spec.whatwg.org/multipage/#concept-script-record>
    /// On failure <https://html.spec.whatwg.org/multipage/#concept-script-error-to-rethrow>
    #[no_trace]
    #[ignore_malloc_size_of = "mozjs"]
    pub record: Result<NonNull<JSScript>, RethrowError>,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-script-fetch-options>
    fetch_options: ScriptFetchOptions,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-base-url>
    #[no_trace]
    url: ServoUrl,
    /// <https://html.spec.whatwg.org/multipage/#muted-errors>
    muted_errors: ErrorReporting,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ErrorReporting {
    Muted,
    Unmuted,
}

impl From<bool> for ErrorReporting {
    fn from(boolean: bool) -> Self {
        if boolean {
            ErrorReporting::Muted
        } else {
            ErrorReporting::Unmuted
        }
    }
}

pub(crate) enum RethrowErrors {
    Yes,
    No,
}

impl GlobalScope {
    /// <https://html.spec.whatwg.org/multipage/#creating-a-classic-script>
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn create_a_classic_script(
        &self,
        source: Cow<'_, str>,
        url: ServoUrl,
        fetch_options: ScriptFetchOptions,
        muted_errors: ErrorReporting,
        introduction_type: Option<&'static CStr>,
        line_number: u32,
        external: bool,
    ) -> ClassicScript {
        let cx = GlobalScope::get_cx();

        let mut script_source = ModuleSource {
            source: Rc::new(DOMString::from(source)),
            unminified_dir: self.unminified_js_dir(),
            external,
            url: url.clone(),
        };
        unminify_js(&mut script_source);

        rooted!(in(*cx) let mut compiled_script = std::ptr::null_mut::<JSScript>());

        // TODO Step 1. If mutedErrors is true, then set baseURL to about:blank.

        // TODO Step 2. If scripting is disabled for settings, then set source to the empty string.

        // TODO Step 4. Set script's settings object to settings.

        // TODO Step 9. Record classic script creation time given script and sourceURLForWindowScripts.

        // Step 10. Let result be ParseScript(source, settings's realm, script).
        compiled_script.set(compile_script(
            cx,
            &script_source.source.str(),
            url.as_str(),
            line_number,
            introduction_type,
        ));

        // Step 11. If result is a list of errors, then:
        let record = if compiled_script.get().is_null() {
            // Step 11.1. Set script's parse error and its error to rethrow to result[0].
            // Step 11.2. Return script.
            Err(RethrowError::from_pending_exception(cx))
        } else {
            Ok(NonNull::new(*compiled_script).expect("Can't be null"))
        };

        // Step 3. Let script be a new classic script that this algorithm will subsequently initialize.
        // Step 5. Set script's base URL to baseURL.
        // Step 6. Set script's fetch options to options.
        // Step 7. Set script's muted errors to mutedErrors.
        // Step 12. Set script's record to result.
        // Step 13. Return script.
        ClassicScript {
            record,
            url,
            fetch_options,
            muted_errors,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-classic-script>
    #[expect(unsafe_code)]
    pub(crate) fn run_a_classic_script(
        &self,
        script: ClassicScript,
        rethrow_errors: RethrowErrors,
        can_gc: CanGc,
    ) -> ErrorResult {
        let cx = GlobalScope::get_cx();
        // TODO Step 1. Let settings be the settings object of script.

        // Step 2. Check if we can run script with settings. If this returns "do not run", then return NormalCompletion(empty).
        if !self.can_run_script() {
            return Ok(());
        }

        // TODO Step 3. Record classic script execution start time given script.

        // Step 4. Prepare to run script given settings.
        // Once dropped this will run "Step 9. Clean up after running script" steps
        let _aes = AutoEntryScript::new(self);

        // Step 5. Let evaluationStatus be null.
        rooted!(in(*cx) let mut evaluation_status = UndefinedValue());
        let mut result = false;

        match script.record {
            // Step 6. If script's error to rethrow is not null, then set evaluationStatus to ThrowCompletion(script's error to rethrow).
            Err(error_to_rethrow) => unsafe {
                JS_SetPendingException(
                    *cx,
                    error_to_rethrow.handle(),
                    ExceptionStackBehavior::Capture,
                )
            },
            // Step 7. Otherwise, set evaluationStatus to ScriptEvaluation(script's record).
            Ok(compiled_script) => {
                rooted!(in(*cx) let mut rval = UndefinedValue());
                result = evaluate_script(
                    cx,
                    compiled_script,
                    script.url,
                    script.fetch_options,
                    rval.handle_mut(),
                );
            },
        }

        unsafe { JS_GetPendingException(*cx, evaluation_status.handle_mut()) };

        // Step 8. If evaluationStatus is an abrupt completion, then:
        if !evaluation_status.is_undefined() {
            warn!("Error evaluating script");

            match (rethrow_errors, script.muted_errors) {
                // Step 8.1. If rethrow errors is true and script's muted errors is false, then:
                (RethrowErrors::Yes, ErrorReporting::Unmuted) => {
                    // Rethrow evaluationStatus.[[Value]].
                    return Err(Error::JSFailed);
                },
                // Step 8.2. If rethrow errors is true and script's muted errors is true, then:
                (RethrowErrors::Yes, ErrorReporting::Muted) => {
                    unsafe { JS_ClearPendingException(*cx) };
                    // Throw a "NetworkError" DOMException.
                    return Err(Error::Network(None));
                },
                // Step 8.3. Otherwise, rethrow errors is false. Perform the following steps:
                _ => {
                    unsafe { JS_ClearPendingException(*cx) };
                    // Report an exception given by evaluationStatus.[[Value]] for script's settings object's global object.
                    self.report_an_exception(cx, evaluation_status.handle(), can_gc);

                    // Return evaluationStatus.
                    return Err(Error::JSFailed);
                },
            }
        }

        maybe_resume_unwind();

        // Step 10. If evaluationStatus is a normal completion, then return evaluationStatus.
        if result {
            return Ok(());
        }

        // Step 11. If we've reached this point, evaluationStatus was left as null because the script
        // was aborted prematurely during evaluation. Return ThrowCompletion(a new QuotaExceededError).
        Err(Error::QuotaExceeded {
            quota: None,
            requested: None,
        })
    }
    pub(crate) fn run_a_module_script(
        &self,
        module_tree: Rc<ModuleTree>,
        _rethrow_errors: bool,
        can_gc: CanGc,
    ) {
        // TODO Step 1. Let settings be the settings object of script.

        // Step 2
        if !self.can_run_script() {
            return;
        }

        // Step 4
        let _aes = AutoEntryScript::new(self);

        // Step 6.
        {
            let module_error = module_tree.get_rethrow_error().borrow();
            if module_error.is_some() {
                module_tree.report_error(self, can_gc);
                return;
            }
        }

        let record = module_tree.get_record().map(|record| record.handle());

        if let Some(record) = record {
            rooted!(in(*GlobalScope::get_cx()) let mut rval = UndefinedValue());
            let evaluated = module_tree.execute_module(self, record, rval.handle_mut(), can_gc);

            if let Err(exception) = evaluated {
                module_tree.set_rethrow_error(exception);
                module_tree.report_error(self, can_gc);
            }
        }
    }
    /// <https://html.spec.whatwg.org/multipage/#check-if-we-can-run-script>
    fn can_run_script(&self) -> bool {
        // Step 1 If the global object specified by settings is a Window object
        // whose Document object is not fully active, then return "do not run".
        //
        // Step 2 If scripting is disabled for settings, then return "do not run".
        //
        // An user agent can also disable scripting
        //
        // Either settings's global object is not a Window object,
        // or settings's global object's associated Document's active sandboxing flag set
        // does not have its sandboxed scripts browsing context flag set.
        if let Some(window) = self.downcast::<Window>() {
            let doc = window.Document();
            doc.is_fully_active() ||
                !doc.has_active_sandboxing_flag(
                    SandboxingFlagSet::SANDBOXED_SCRIPTS_BROWSING_CONTEXT_FLAG,
                )
        } else {
            true
        }
    }
}

#[expect(unsafe_code)]
pub(crate) fn compile_script(
    cx: SafeJSContext,
    text: &str,
    filename: &str,
    line_number: u32,
    introduction_type: Option<&'static CStr>,
) -> *mut JSScript {
    let mut options = unsafe { CompileOptionsWrapper::new_raw(*cx, filename, line_number) };
    if let Some(introduction_type) = introduction_type {
        options.set_introduction_type(introduction_type);
    }

    debug!("Compiling script");
    unsafe { Compile1(*cx, options.ptr, &mut transform_str_to_source_text(text)) }
}

/// <https://tc39.es/ecma262/#sec-runtime-semantics-scriptevaluation>
#[expect(unsafe_code)]
pub(crate) fn evaluate_script(
    cx: SafeJSContext,
    compiled_script: NonNull<JSScript>,
    url: ServoUrl,
    fetch_options: ScriptFetchOptions,
    rval: MutableHandleValue,
) -> bool {
    rooted!(in(*cx) let record = compiled_script.as_ptr());
    rooted!(in(*cx) let mut script_private = UndefinedValue());

    unsafe { JS_GetScriptPrivate(*record, script_private.handle_mut()) };

    // When `ScriptPrivate` for the compiled script is undefined,
    // we need to set it so that it can be used in dynamic import context.
    if script_private.is_undefined() {
        debug!("Set script private for {}", url);
        let module_script_data = Rc::new(ModuleScript::new(
            url,
            fetch_options,
            // We can't initialize an module owner here because
            // the executing context of script might be different
            // from the dynamic import script's executing context.
            None,
        ));

        unsafe {
            SetScriptPrivate(
                *record,
                &PrivateValue(Rc::into_raw(module_script_data) as *const _),
            );
        }
    }

    unsafe { JS_ExecuteScript(*cx, record.handle(), rval) }
}
