/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;

use crate::dom::bindings::codegen::Bindings::WorkerNavigatorBinding::WorkerNavigatorMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::navigator::hardware_concurrency;
use crate::dom::navigatorinfo;
use crate::dom::permissions::Permissions;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpu::GPU;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::script_runtime::{CanGc, JSContext};

// https://html.spec.whatwg.org/multipage/#workernavigator
#[dom_struct]
pub(crate) struct WorkerNavigator {
    reflector_: Reflector,
    permissions: MutNullableDom<Permissions>,
    #[cfg(feature = "webgpu")]
    gpu: MutNullableDom<GPU>,
}

impl WorkerNavigator {
    fn new_inherited() -> WorkerNavigator {
        WorkerNavigator {
            reflector_: Reflector::new(),
            permissions: Default::default(),
            #[cfg(feature = "webgpu")]
            gpu: Default::default(),
        }
    }

    pub(crate) fn new(global: &WorkerGlobalScope, can_gc: CanGc) -> DomRoot<WorkerNavigator> {
        reflect_dom_object(Box::new(WorkerNavigator::new_inherited()), global, can_gc)
    }
}

impl WorkerNavigatorMethods<crate::DomTypeHolder> for WorkerNavigator {
    // https://html.spec.whatwg.org/multipage/#dom-navigator-product
    fn Product(&self) -> DOMString {
        navigatorinfo::Product()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-productsub
    fn ProductSub(&self) -> DOMString {
        navigatorinfo::ProductSub()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-vendor
    fn Vendor(&self) -> DOMString {
        navigatorinfo::Vendor()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-vendorsub
    fn VendorSub(&self) -> DOMString {
        navigatorinfo::VendorSub()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-taintenabled
    fn TaintEnabled(&self) -> bool {
        navigatorinfo::TaintEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appname
    fn AppName(&self) -> DOMString {
        navigatorinfo::AppName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appcodename
    fn AppCodeName(&self) -> DOMString {
        navigatorinfo::AppCodeName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-platform
    fn Platform(&self) -> DOMString {
        navigatorinfo::Platform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-useragent
    fn UserAgent(&self) -> DOMString {
        navigatorinfo::UserAgent(self.global().get_user_agent())
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appversion
    fn AppVersion(&self) -> DOMString {
        navigatorinfo::AppVersion()
    }

    // https://html.spec.whatwg.org/multipage/#navigatorlanguage
    fn Language(&self) -> DOMString {
        navigatorinfo::Language()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-languages
    #[allow(unsafe_code)]
    fn Languages(&self, cx: JSContext, retval: MutableHandleValue) {
        to_frozen_array(&[self.Language()], cx, retval)
    }

    // https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
    fn Permissions(&self) -> DomRoot<Permissions> {
        self.permissions
            .or_init(|| Permissions::new(&self.global(), CanGc::note()))
    }

    // https://gpuweb.github.io/gpuweb/#dom-navigator-gpu
    #[cfg(feature = "webgpu")]
    fn Gpu(&self) -> DomRoot<GPU> {
        self.gpu.or_init(|| GPU::new(&self.global(), CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-hardwareconcurrency>
    fn HardwareConcurrency(&self) -> u64 {
        hardware_concurrency()
    }
}
