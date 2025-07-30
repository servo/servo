/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject as SafeHandleObject;

use crate::dom::bindings::codegen::Bindings::TextEncoderStreamBinding::TextEncoderStreamMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::script_runtime::CanGc;
use crate::{DomTypeHolder, DomTypes};

/// <https://encoding.spec.whatwg.org/#textencoderstream>
#[dom_struct]
pub(crate) struct TextEncoderStream {
    reflector_: Reflector,
}

#[allow(non_snake_case)]
impl TextEncoderStreamMethods<DomTypeHolder> for TextEncoderStream {
    /// <https://encoding.spec.whatwg.org/#dom-textencoderstream>
    fn Constructor(
        r#global: &<DomTypeHolder as DomTypes>::GlobalScope,
        r#proto: Option<SafeHandleObject>,
        r#can_gc: CanGc,
    ) -> Fallible<DomRoot<<DomTypeHolder as DomTypes>::TextEncoderStream>> {
        todo!()
    }

    /// <https://encoding.spec.whatwg.org/#dom-textencoder-encoding>
    fn Encoding(&self) -> DOMString {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-readable>
    fn Readable(&self) -> DomRoot<<DomTypeHolder as script_bindings::DomTypes>::ReadableStream> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-writable>
    fn Writable(&self) -> DomRoot<<DomTypeHolder as DomTypes>::WritableStream> {
        todo!()
    }
}
