/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BlobBinding::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestMethods;
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::conversions::IDLInterface;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::MutDomObject;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::blob::Blob;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom::formdata::FormData;
use dom::globalscope::GlobalScope;
use dom::htmlformelement::{encode_multipart_form_data, generate_boundary};
use dom::node::Node;
use dom::urlsearchparams::URLSearchParams;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use encoding_rs::UTF_8;
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use malloc_size_of::MallocSizeOf;
use std::borrow::ToOwned;
use std::default::Default;
use typeholder::TypeHolderTrait;

pub trait XMLHttpRequestTrait<TH: TypeHolderTrait>:
    MutDomObject
    + IDLInterface
    + MallocSizeOf
    + JSTraceable
    + XMLHttpRequestMethods<TH>
    + Castable
    + DerivedFrom<EventTarget<TH>>
    + DerivedFrom<XMLHttpRequestEventTarget<TH>>
{
    // https://xhr.spec.whatwg.org/#constructors
    fn Constructor(global: &GlobalScope<TH>) -> Fallible<DomRoot<TH::XMLHttpRequest>>;
}

pub trait XHRTimeoutCallbackTrait<TH: TypeHolderTrait>: JSTraceable + MallocSizeOf {
    fn invoke(self);
}

pub trait Extractable {
    fn extract(&self) -> (Vec<u8>, Option<DOMString>);
}

impl<TH: TypeHolderTrait> Extractable for Blob<TH> {
    fn extract(&self) -> (Vec<u8>, Option<DOMString>) {
        let content_type = if self.Type().as_ref().is_empty() {
            None
        } else {
            Some(self.Type())
        };
        let bytes = self.get_bytes().unwrap_or(vec![]);
        (bytes, content_type)
    }
}

impl Extractable for DOMString {
    fn extract(&self) -> (Vec<u8>, Option<DOMString>) {
        (
            self.as_bytes().to_owned(),
            Some(DOMString::from("text/plain;charset=UTF-8")),
        )
    }
}

impl<TH: TypeHolderTrait> Extractable for FormData<TH> {
    fn extract(&self) -> (Vec<u8>, Option<DOMString>) {
        let boundary = generate_boundary();
        let bytes = encode_multipart_form_data(&mut self.datums(), boundary.clone(), UTF_8);
        (
            bytes,
            Some(DOMString::from(format!(
                "multipart/form-data;boundary={}",
                boundary
            ))),
        )
    }
}

impl<TH: TypeHolderTrait> Extractable for URLSearchParams<TH> {
    fn extract(&self) -> (Vec<u8>, Option<DOMString>) {
        (
            self.serialize_utf8().into_bytes(),
            Some(DOMString::from(
                "application/x-www-form-urlencoded;charset=UTF-8",
            )),
        )
    }
}

fn serialize_document<TH: TypeHolderTrait>(doc: &Document<TH>) -> Fallible<DOMString> {
    let mut writer = vec![];
    match serialize(
        &mut writer,
        &doc.upcast::<Node<TH>>(),
        SerializeOpts::default(),
    ) {
        Ok(_) => Ok(DOMString::from(String::from_utf8(writer).unwrap())),
        Err(_) => Err(Error::InvalidState),
    }
}

impl<TH: TypeHolderTrait> Extractable for BodyInit<TH> {
    // https://fetch.spec.whatwg.org/#concept-bodyinit-extract
    fn extract(&self) -> (Vec<u8>, Option<DOMString>) {
        match *self {
            BodyInit::String(ref s) => s.extract(),
            BodyInit::URLSearchParams(ref usp) => usp.extract(),
            BodyInit::Blob(ref b) => b.extract(),
            BodyInit::FormData(ref formdata) => formdata.extract(),
            BodyInit::ArrayBuffer(ref typedarray) => ((typedarray.to_vec(), None)),
            BodyInit::ArrayBufferView(ref typedarray) => ((typedarray.to_vec(), None)),
        }
    }
}

/// Returns whether `bs` is a `field-value`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-32).
pub fn is_field_value(slice: &[u8]) -> bool {
    // Classifications of characters necessary for the [CRLF] (SP|HT) rule
    #[derive(PartialEq)]
    enum PreviousCharacter {
        Other,
        CR,
        LF,
        SPHT, // SP or HT
    }
    let mut prev = PreviousCharacter::Other; // The previous character
    slice.iter().all(|&x| {
        // http://tools.ietf.org/html/rfc2616#section-2.2
        match x {
            13 => {
                // CR
                if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::CR;
                    true
                } else {
                    false
                }
            },
            10 => {
                // LF
                if prev == PreviousCharacter::CR {
                    prev = PreviousCharacter::LF;
                    true
                } else {
                    false
                }
            },
            32 => {
                // SP
                if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::SPHT;
                    true
                } else if prev == PreviousCharacter::Other {
                    // Counts as an Other here, since it's not preceded by a CRLF
                    // SP is not a CTL, so it can be used anywhere
                    // though if used immediately after a CR the CR is invalid
                    // We don't change prev since it's already Other
                    true
                } else {
                    false
                }
            },
            9 => {
                // HT
                if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::SPHT;
                    true
                } else {
                    false
                }
            },
            0...31 | 127 => false, // CTLs
            x if x > 127 => false, // non ASCII
            _ if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT => {
                prev = PreviousCharacter::Other;
                true
            },
            _ => false, // Previous character was a CR/LF but not part of the [CRLF] (SP|HT) rule
        }
    })
}
