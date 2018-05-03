/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::VRDisplayCapabilitiesBinding;
use dom::bindings::codegen::Bindings::VRDisplayCapabilitiesBinding::VRDisplayCapabilitiesMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webvr_traits::WebVRDisplayCapabilities;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct VRDisplayCapabilities<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    capabilities: DomRefCell<WebVRDisplayCapabilities>,
    _p: PhantomData<TH>,
}

unsafe_no_jsmanaged_fields!(WebVRDisplayCapabilities);

impl<TH: TypeHolderTrait> VRDisplayCapabilities<TH> {
    fn new_inherited(capabilities: WebVRDisplayCapabilities) -> VRDisplayCapabilities<TH> {
        VRDisplayCapabilities {
            reflector_: Reflector::new(),
            capabilities: DomRefCell::new(capabilities),
            _p: Default::default(),
        }
    }

    pub fn new(capabilities: WebVRDisplayCapabilities, global: &GlobalScope<TH>) -> DomRoot<VRDisplayCapabilities<TH>> {
        reflect_dom_object(Box::new(VRDisplayCapabilities::new_inherited(capabilities)),
                           global,
                           VRDisplayCapabilitiesBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> VRDisplayCapabilitiesMethods for VRDisplayCapabilities<TH> {
    // https://w3c.github.io/webvr/#dom-vrdisplaycapabilities-hasposition
    fn HasPosition(&self) -> bool {
        self.capabilities.borrow().has_position
    }

    // https://w3c.github.io/webvr/#dom-vrdisplaycapabilities-hasorientation
    fn HasOrientation(&self) -> bool {
        self.capabilities.borrow().has_orientation
    }

    // https://w3c.github.io/webvr/#dom-vrdisplaycapabilities-hasexternaldisplay
    fn HasExternalDisplay(&self) -> bool {
        self.capabilities.borrow().has_external_display
    }

    // https://w3c.github.io/webvr/#dom-vrdisplaycapabilities-canpresent
    fn CanPresent(&self) -> bool {
        self.capabilities.borrow().can_present
    }

    // https://w3c.github.io/webvr/#dom-vrdisplaycapabilities-maxlayers
    fn MaxLayers(&self) -> u32 {
        if self.CanPresent() { 1 } else { 0 }
    }
}
