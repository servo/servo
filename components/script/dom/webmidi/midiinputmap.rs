/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use indexmap::IndexMap;
use script_bindings::cell::DomRefCell;
use script_bindings::like::Maplike;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::MIDIInputMapBinding::MIDIInputMapMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webmidi::midiinput::MIDIInput;
use crate::maplike;

// https://webaudio.github.io/web-midi-api/#MIDIInputMap
#[dom_struct]
pub(crate) struct MIDIInputMap {
    reflector_: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexMap<DOMString, DomRoot<MIDIInput>>>,
}

impl MIDIInputMap {
    fn new_inherited() -> MIDIInputMap {
        MIDIInputMap {
            reflector_: Reflector::new(),
            internal: DomRefCell::new(IndexMap::new()),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<MIDIInputMap> {
        reflect_dom_object_with_cx(Box::new(MIDIInputMap::new_inherited()), global, cx)
    }
}

impl MIDIInputMapMethods<crate::DomTypeHolder> for MIDIInputMap {
    fn Size(&self, cx: &mut js::context::JSContext) -> u32 {
        self.internal.size(cx)
    }
}

impl Maplike for MIDIInputMap {
    type Key = DOMString;
    type Value = DomRoot<MIDIInput>;

    maplike!(self, internal);
}
