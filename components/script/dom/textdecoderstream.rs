/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::root::DomRoot;
use js::rust::HandleObject;

use crate::DomTypes;
use crate::dom::bindings::codegen::Bindings::TextDecoderStreamBinding::TextDecoderStreamMethods;
use crate::dom::bindings::codegen::Bindings::TextDecoderBinding;
use crate::dom::bindings::reflector::{Reflector};
use crate::dom::bindings::str::{DOMString};
use crate::dom::bindings::error::{Fallible};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct TextDecoderStream {
    reflector_: Reflector,
}

#[allow(non_snake_case)]
impl TextDecoderStreamMethods<crate::DomTypeHolder> for TextDecoderStream {
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        label: DOMString,
        options: &TextDecoderBinding::TextDecoderOptions
    ) -> Fallible<DomRoot<TextDecoderStream>> {
        todo!()
    }

    fn Readable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::ReadableStream> {
        todo!()
    }

    fn Writable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::WritableStream> {
        todo!()
    }

    fn Encoding(&self) -> DOMString {
        todo!()
    }

    fn Fatal(&self) -> bool {
        todo!()
    }

    fn IgnoreBOM(&self) -> bool {
        todo!()
    }
}

