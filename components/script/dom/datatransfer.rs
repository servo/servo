/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferBinding::DataTransferMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::element::Element;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct DataTransfer {
    reflector_: Reflector,
    drop_effect: DomRefCell<DOMString>,
    effect_allowed: DomRefCell<DOMString>,
    items: Dom<DataTransferItemList>,
}

impl DataTransferMethods for DataTransfer {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DataTransfer> {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect>
    fn DropEffect(&self) -> DOMString {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-dropeffect>
    fn SetDropEffect(&self, value: DOMString) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed>
    fn EffectAllowed(&self) -> DOMString {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-effectallowed>
    fn SetEffectAllowed(&self, value: DOMString) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-items>
    fn Items(&self) -> DomRoot<DataTransferItemList> {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransfer-setdragimage>
    fn SetDragImage(&self, image: &Element, x: i32, y: i32) {
        todo!()
    }
}
