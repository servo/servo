/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::USVString;
use dom::blob::{Blob, BlobImpl};
use dom::formdata::FormData;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use js::jsapi::JSContext;
use js::jsapi::JS_ClearPendingException;
use js::jsapi::JS_ParseJSON;
use js::jsapi::Value as JSValue;
use js::jsval::UndefinedValue;
use mime::{Mime, TopLevel, SubLevel};
use std::cell::Ref;
use std::rc::Rc;
use std::str;
use url::form_urlencoded;

#[derive(Copy, Clone, JSTraceable, HeapSizeOf)]
pub enum BodyType {
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
pub fn consume_body<T: BodyOperations + DomObject>(object: &T, body_type: BodyType) -> Rc<Promise> {
    let promise = Promise::new(&object.global());

    // Step 1
    if object.get_body_used() || object.is_locked() {
        promise.reject_error(promise.global().get_cx(), Error::Type(
            "The response's stream is disturbed or locked".to_string()));
        return promise;
    }

    object.set_body_promise(&promise, body_type);

    // Steps 2-4
    // TODO: Body does not yet have a stream.

    consume_body_with_promise(object, body_type, &promise);

    promise
}

// https://fetch.spec.whatwg.org/#concept-body-consume-body
#[allow(unrooted_must_root)]
pub fn consume_body_with_promise<T: BodyOperations + DomObject>(object: &T,
                                                                body_type: BodyType,
                                                                promise: &Promise) {
    // Step 5
    let body = match object.take_body() {
        Some(body) => body,
        None => return,
    };

    let pkg_data_results = run_package_data_algorithm(object,
                                                      body,
                                                      body_type,
                                                      object.get_mime_type());

    let cx = promise.global().get_cx();
    match pkg_data_results {
        Ok(results) => {
            match results {
                FetchedData::Text(s) => promise.resolve_native(cx, &USVString(s)),
                FetchedData::Json(j) => promise.resolve_native(cx, &j),
                FetchedData::BlobData(b) => promise.resolve_native(cx, &b),
                FetchedData::FormData(f) => promise.resolve_native(cx, &f),
            };
        },
        Err(err) => promise.reject_error(cx, err),
    }
}

// https://fetch.spec.whatwg.org/#concept-body-package-data
#[allow(unsafe_code)]
fn run_package_data_algorithm<T: BodyOperations + DomObject>(object: &T,
                                                             bytes: Vec<u8>,
                                                             body_type: BodyType,
                                                             mime_type: Ref<Vec<u8>>)
                                                             -> Fallible<FetchedData> {
    let global = object.global();
    let cx = global.get_cx();
    let mime = &*mime_type;
    match body_type {
        BodyType::Text => run_text_data_algorithm(bytes),
        BodyType::Json => run_json_data_algorithm(cx, bytes),
        BodyType::Blob => run_blob_data_algorithm(&global, bytes, mime),
        BodyType::FormData => run_form_data_algorithm(&global, bytes, mime),
    }
}

fn run_text_data_algorithm(bytes: Vec<u8>) -> Fallible<FetchedData> {
    Ok(FetchedData::Text(String::from_utf8_lossy(&bytes).into_owned()))
}

#[allow(unsafe_code)]
fn run_json_data_algorithm(cx: *mut JSContext,
                           bytes: Vec<u8>) -> Fallible<FetchedData> {
    let json_text = String::from_utf8_lossy(&bytes);
    let json_text: Vec<u16> = json_text.encode_utf16().collect();
    rooted!(in(cx) let mut rval = UndefinedValue());
    unsafe {
        if !JS_ParseJSON(cx,
                         json_text.as_ptr(),
                         json_text.len() as u32,
                         rval.handle_mut()) {
            JS_ClearPendingException(cx);
            // TODO: See issue #13464. Exception should be thrown instead of cleared.
            return Err(Error::Type("Failed to parse JSON".to_string()));
        }
        Ok(FetchedData::Json(rval.get()))
    }
}

fn run_blob_data_algorithm(root: &GlobalScope,
                           bytes: Vec<u8>,
                           mime: &[u8]) -> Fallible<FetchedData> {
    let mime_string = if let Ok(s) = String::from_utf8(mime.to_vec()) {
        s
    } else {
        "".to_string()
    };
    let blob = Blob::new(root, BlobImpl::new_from_bytes(bytes), mime_string);
    Ok(FetchedData::BlobData(blob))
}

fn run_form_data_algorithm(root: &GlobalScope, bytes: Vec<u8>, mime: &[u8]) -> Fallible<FetchedData> {
    let mime_str = if let Ok(s) = str::from_utf8(mime) {
        s
    } else {
        ""
    };
    let mime: Mime = mime_str.parse().map_err(
        |_| Error::Type("Inappropriate MIME-type for Body".to_string()))?;
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

pub trait BodyOperations {
    fn get_body_used(&self) -> bool;
    fn set_body_promise(&self, p: &Rc<Promise>, body_type: BodyType);
    /// Returns `Some(_)` if the body is complete, `None` if there is more to
    /// come.
    fn take_body(&self) -> Option<Vec<u8>>;
    fn is_locked(&self) -> bool;
    fn get_mime_type(&self) -> Ref<Vec<u8>>;
}
