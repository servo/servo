/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextDecoderBinding;
use dom::bindings::codegen::Bindings::TextDecoderBinding::TextDecoderMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflector, reflect_dom_object};

use util::str::DOMString;

use encoding::Encoding;
use encoding::types::EncodingRef;
use encoding::label::encoding_from_whatwg_label;

use std::borrow::ToOwned;

#[dom_struct]
pub struct TextDecoder {
    reflector_: Reflector,
    encoding: EncodingRef,
    fatal: bool,
}

impl TextDecoder {
    fn new_inherited(encoding: EncodingRef, fatal: bool) -> TextDecoder {
        TextDecoder {
            reflector_: Reflector::new(),
            encoding: encoding,
            fatal: fatal,
        }
    }

    pub fn new(global: GlobalRef, encoding: EncodingRef, fatal: bool) -> Temporary<TextDecoder> {
        reflect_dom_object(box TextDecoder::new_inherited(encoding, fatal),
                           global,
                           TextDecoderBinding::Wrap)
    }

    /// https://encoding.spec.whatwg.org/#dom-textdecoder
    pub fn Constructor(global: GlobalRef,
                       label: DOMString,
                       options: &TextDecoderBinding::TextDecoderOptions)
                            -> Fallible<Temporary<TextDecoder>> {
        let encoding = match encoding_from_whatwg_label(&label) {
            Some(enc) => enc,
            // FIXME: Should throw a RangeError as per spec
            None => return Err(Error::Syntax),
        };
        Ok(TextDecoder::new(global, encoding, options.fatal))
    }
}

impl<'a> TextDecoderMethods for JSRef<'a, TextDecoder> {
    fn Encoding(self) -> DOMString {
        self.encoding.whatwg_name().unwrap().to_owned()
    }

    fn Fatal(self) -> bool {
        self.fatal
    }
}
