/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PerformanceNavigationBinding::{
    PerformanceNavigationConstants, PerformanceNavigationMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PerformanceNavigation {
    reflector_: Reflector,
}

impl PerformanceNavigation {
    fn new_inherited() -> PerformanceNavigation {
        PerformanceNavigation {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<PerformanceNavigation> {
        reflect_dom_object(
            Box::new(PerformanceNavigation::new_inherited()),
            global,
            CanGc::note(),
        )
    }
}

impl PerformanceNavigationMethods<crate::DomTypeHolder> for PerformanceNavigation {
    // https://w3c.github.io/navigation-timing/#dom-performancenavigation-type
    fn Type(&self) -> u16 {
        PerformanceNavigationConstants::TYPE_NAVIGATE
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigation-redirectcount
    fn RedirectCount(&self) -> u16 {
        self.global().as_window().Document().get_redirect_count()
    }
}
