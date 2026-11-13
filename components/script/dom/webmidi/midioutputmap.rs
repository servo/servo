/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use indexmap::IndexMap;
use script_bindings::cell::DomRefCell;
use script_bindings::like::Maplike;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::codegen::Bindings::MIDIOutputMapBinding::MIDIOutputMapMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webmidi::midioutput::MIDIOutput;
use crate::maplike;

// https://webaudio.github.io/web-midi-api/#MIDIOutputMap
#[dom_struct]
pub(crate) struct MIDIOutputMap {
    reflector_: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexMap<DOMString, DomRoot<MIDIOutput>>>,
}

impl MIDIOutputMap {
    fn new_inherited() -> MIDIOutputMap {
        MIDIOutputMap {
            reflector_: Reflector::new(),
            internal: DomRefCell::new(IndexMap::with_capacity(64)),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
    ) -> DomRoot<MIDIOutputMap> {
        reflect_dom_object(cx, Box::new(MIDIOutputMap::new_inherited()), global)
    }
}

impl MIDIOutputMapMethods<crate::DomTypeHolder> for MIDIOutputMap {
    fn Size(&self, cx: &mut js::context::JSContext) -> u32 {
        self.internal.size(cx)
    }
}

impl Maplike for MIDIOutputMap {
    type Key = DOMString;
    type Value = DomRoot<MIDIOutput>;

    maplike!(self, internal);
}
