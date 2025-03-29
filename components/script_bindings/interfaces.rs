/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::rust::HandleObject;

use crate::script_runtime::JSContext;

pub trait DocumentHelpers {
    fn ensure_safe_to_run_script_or_layout(&self);
}

pub trait ServoInternalsHelpers {
    fn is_servo_internal(cx: JSContext, global: HandleObject) -> bool;
}

pub trait TestBindingHelpers {
    fn condition_satisfied(cx: JSContext, global: HandleObject) -> bool;
    fn condition_unsatisfied(cx: JSContext, global: HandleObject) -> bool;
}

pub trait WebGL2RenderingContextHelpers {
    fn is_webgl2_enabled(cx: JSContext, global: HandleObject) -> bool;
}
