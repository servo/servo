/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use backtrace::Backtrace;
use dom_struct::dom_struct;
use js::context::JSContext;
use layout_api::ReflowPhasesRun;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::domstring::DOMString;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use time::Duration;

use crate::dom::bindings::codegen::Bindings::ServoTestUtilsBinding::ServoTestUtilsMethods;
use crate::dom::globalscope::GlobalScope;
use crate::dom::layoutresult::LayoutResult;

#[dom_struct]
pub(crate) struct ServoTestUtils {
    reflector_: Reflector,
}

impl ServoTestUtilsMethods<crate::DomTypeHolder> for ServoTestUtils {
    fn AdvanceClock(global: &GlobalScope, ms: i32) {
        global
            .as_window()
            .advance_animation_clock(Duration::milliseconds(ms as i64));
    }

    #[expect(unsafe_code)]
    fn CrashHard(_: &GlobalScope) {
        unsafe { std::ptr::null_mut::<i32>().write(42) }
    }

    fn ForceLayout(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<LayoutResult> {
        let (phases_run, statistics) = global.as_window().Document().update_the_rendering(cx);

        let mut phases = Vec::new();
        if phases_run.contains(ReflowPhasesRun::RanLayout) {
            phases.push(DOMString::from("RanLayout"))
        }
        if phases_run.contains(ReflowPhasesRun::BuiltStackingContextTree) {
            phases.push(DOMString::from("BuiltStackingContextTree"))
        }
        if phases_run.contains(ReflowPhasesRun::BuiltDisplayList) {
            phases.push(DOMString::from("BuiltDisplayList"))
        }
        if phases_run.contains(ReflowPhasesRun::UpdatedScrollNodeOffset) {
            phases.push(DOMString::from("UpdatedScrollNodeOffset"))
        }
        if phases_run.contains(ReflowPhasesRun::UpdatedImageData) {
            phases.push(DOMString::from("UpdatedImageData"))
        }

        LayoutResult::new(
            cx,
            global,
            phases,
            statistics.rebuilt_fragment_count,
            statistics.restyle_fragment_count,
            statistics.only_descendants_changed_count,
        )
    }

    fn Js_backtrace(_: &GlobalScope) {
        println!("Current JS stack:");
        let rust_stack = Backtrace::new();
        println!("Current Rust stack:\n{:?}", rust_stack);
    }

    fn Panic(_: &GlobalScope) {
        panic!("explicit panic from script")
    }

    fn ForceAccessibilityUpdate(cx: &mut JSContext, global: &GlobalScope) {
        let window = global.as_window();
        window.layout().set_needs_accessibility_update();
        let _ = window.Document().update_the_rendering(cx);
    }
}
