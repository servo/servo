/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use crate::dom::bindings::codegen::Bindings::TestBindingBinding::TestNS_Binding;
use crate::dom::globalscope::GlobalScope; 
use crate::dom::testbinding::TestBinding;
use crate::dom::bindings::root::DomRoot;
use js::context::JSContext;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TestNS(());

impl TestNS_Binding::TestNSMethods<crate::DomTypeHolder> for TestNS {
    fn TestAttribute(cx: &mut JSContext, global_scope: &GlobalScope) -> DomRoot<TestBinding> {
        TestBinding::new(cx, global_scope, None)
    }
}
