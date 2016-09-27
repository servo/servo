/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::USVString;
use dom::blob::{Blob, BlobImpl};
use dom::formdata::FormData;
use dom::promise::Promise;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use js::jsapi::JSContext;
use js::jsapi::JS_ClearPendingException;
use js::jsapi::JS_ParseJSON;
use js::jsapi::Value as JSValue;
use js::jsval::UndefinedValue;
use mime::{Mime, TopLevel, SubLevel};
use std::rc::Rc;
use std::str;
use style::refcell::Ref;
use url::form_urlencoded;

pub enum BodyType {
    ArrayBuffer,
    Blob,
    FormData,
    Json,
    Text
}

pub enum FetchedData {
    Text(String),
    Json(JSValue),
    BlobData(Root<Blob>),
    FormData(Root<FormData>),
}

// https://fetch.spec.whatwg.org/#concept-body-consume-body
#[allow(unrooted_must_root)]
pub fn consume_body<T: BodyTrait + Reflectable>(object: &T, body_type: BodyType) -> Rc<Promise> {
    let promise = Promise::new(object.global().r());

    // Step 1
    if object.get_body_used_trait() || object.locked_trait() {
        promise.reject_error(promise.global().r().get_cx(), Error::Type(
            "The response's stream is disturbed or locked".to_string()));
    }

    // Steps 2-4
    // TODO: Body does not yet have a stream.

    // Step 5
    let pkg_data_results = run_package_data_algorithm(object,
                                                      object.take_body(),
                                                      body_type,
                                                      object.get_mime_type());
    match pkg_data_results {
        Err(e) => promise.reject_error(promise.global().r().get_cx(), e),
        Ok(r) => {
            match r {
                FetchedData::Text(s) => promise.resolve_native(promise.global().r().get_cx(), &USVString(s)),
                FetchedData::Json(j) => promise.resolve_native(promise.global().r().get_cx(), &j),
                FetchedData::BlobData(b) => promise.resolve_native(promise.global().r().get_cx(), &b),
                FetchedData::FormData(f) => promise.resolve_native(promise.global().r().get_cx(), &f),
            };
        },
    }
    promise
}

// https://fetch.spec.whatwg.org/#concept-body-package-data
#[allow(unsafe_code)]
fn run_package_data_algorithm<T: BodyTrait + Reflectable>(object: &T,
                                                          bytes: Option<Vec<u8>>,
                                                          body_type: BodyType,
                                                          mime_type: Ref<Vec<u8>>) -> Fallible<FetchedData> {
    if let Some(b) = bytes {
        let cx = object.global().r().get_cx();
        let mime = &*mime_type;
        match body_type {
            BodyType::Text => run_text_data_algorithm(b),
            BodyType::Json => run_json_data_algorithm(cx, b),
            BodyType::Blob => run_blob_data_algorithm(object.global().r(), b, mime),
            BodyType::FormData => run_form_data_algorithm(object.global().r(), b, mime),
            _ => Err(Error::Type("Unable to process body type".to_string()))
        }
    } else {
        return Err(Error::Type("Unable to process bytes".to_string()));
    }
}

fn run_text_data_algorithm(bytes: Vec<u8>) -> Fallible<FetchedData> {
    let text = UTF_8.decode(&bytes, DecoderTrap::Replace).unwrap();
    Ok(FetchedData::Text(text))
}

#[allow(unsafe_code)]
fn run_json_data_algorithm(cx: *mut JSContext,
                           bytes: Vec<u8>) -> Fallible<FetchedData> {
    let json_text = UTF_8.decode(&bytes, DecoderTrap::Replace).unwrap();
    let json_text: Vec<u16> = json_text.encode_utf16().collect();
    rooted!(in(cx) let mut rval = UndefinedValue());
    unsafe {
        if !JS_ParseJSON(cx,
                         json_text.as_ptr(),
                         json_text.len() as u32,
                         rval.handle_mut()) {
            JS_ClearPendingException(cx);
            // TODO
            // ... Should this return `NullValue()`?
            return Err(Error::Type("Failed to parse JSON".to_string()));
            }
        Ok(FetchedData::Json(rval.get()))
    }
}

fn run_blob_data_algorithm(root: GlobalRef,
                           bytes: Vec<u8>,
                           mime: &[u8]) -> Fallible<FetchedData> {
    let mime_string = if let Ok(s) = String::from_utf8(mime.to_vec()) {
        s
    } else {
        "None".to_string()
    };
    let blob = Blob::new(root, BlobImpl::new_from_bytes(bytes), mime_string);
    Ok(FetchedData::BlobData(blob))
}

fn run_form_data_algorithm(root: GlobalRef, bytes: Vec<u8>, mime: &[u8]) -> Fallible<FetchedData> {
    let mime_str = if let Ok(s) = str::from_utf8(mime) {
        s
    } else {
        "None"
    };
    let mime: Mime = try!(mime_str.parse().map_err(
        |_| Error::Type("Inappropriate MIME-type for Body".to_string())));
    match mime {
        // TODO
        // ... Parser for Mime(TopLevel::Multipart, SubLevel::FormData, _)
        // ... is not fully determined yet.
        Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, _) => {
            let entries = form_urlencoded::parse(&bytes);
            let formdata = FormData::new(None, root);
            for (k, e) in entries {
                formdata.Append(USVString(k.into_owned()), USVString(e.into_owned()));
            }
            return Ok(FetchedData::FormData(formdata));
        },
        _ => return Err(Error::Type("Inappropriate MIME-type for Body".to_string())),
    }
}

pub trait BodyTrait {
    fn get_body_used_trait(&self) -> bool;
    fn take_body(&self) -> Option<Vec<u8>>;
    fn locked_trait(&self) -> bool;
    fn get_mime_type(&self) -> Ref<Vec<u8>>;
}
