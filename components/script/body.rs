/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::USVString;
use dom::promise::Promise;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use std::rc::Rc;
use style::refcell::Ref;

// https://fetch.spec.whatwg.org/#concept-body-consume-body
#[allow(unrooted_must_root)]
pub fn consume_body<T: BodyTrait + Reflectable>(object: &T, body_type: BodyType) -> Rc<Promise> {
    let promise = Promise::new(object.global().r());

    // Step 1
    if object.get_body_used_trait() || object.locked_trait() {
        promise.maybe_reject_error(promise.global().r().get_cx(), Error::Type(
            "The response's stream is disturbed or locked".to_string()));
    }

    // Steps 2-4
    // TODO: Body does not yet have a stream.

    // Step 5
    let pkg_data_results = run_package_data_algorithm(object.take_body(),
                                                      body_type, object.get_mime_type());
    if pkg_data_results.is_ok() {
        let pkg_data_results = USVString(pkg_data_results.unwrap());
        promise.maybe_resolve_native(promise.global().r().get_cx(), &pkg_data_results);
    } else {
        promise.maybe_reject_error(promise.global().r().get_cx(),
                                   pkg_data_results.unwrap_err());
    }
    promise
}

// https://fetch.spec.whatwg.org/#concept-body-package-data
fn run_package_data_algorithm(bytes: Option<Vec<u8>>,
                              body_type: BodyType, mime_type: Ref<Vec<u8>>) -> Fallible<String> {
        match body_type {
            BodyType::Text => {
                // return result of running utf-8 decode here
                // using encoding crate on all bytes instead of
                // individually processing each token
                if let Some(bytes) = bytes {
                    UTF_8.decode(&bytes, DecoderTrap::Replace).unwrap();
                }
                return Ok("".to_owned());
            },
            _ => Err(Error::Type("Unable to process bytes".to_string()))
        }
}

pub enum BodyType {
    ArrayBuffer,
    Blob,
    FormData,
    Json,
    Text
}

pub trait BodyTrait {
    fn get_body_used_trait(&self) -> bool;
    fn take_body(&self) -> Option<Vec<u8>>;
    fn locked_trait(&self) -> bool;
    fn get_mime_type(&self) -> Ref<Vec<u8>>;
}
