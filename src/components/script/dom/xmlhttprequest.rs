/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::XMLHttpRequestBinding;
use dom::bindings::str::ByteString;
use self::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use self::XMLHttpRequestBinding::XMLHttpRequestResponseTypeValues::_empty;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestDerived;
use dom::document::Document;
use dom::eventtarget::{EventTarget, XMLHttpRequestTargetTypeId};
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable};
use js::jsapi::JSContext;
use js::jsval::{JSVal, NullValue};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;
use servo_util::str::DOMString;

#[deriving(Eq,Encodable)]
pub enum XMLHttpRequestId {
    XMLHttpRequestTypeId,
    XMLHttpRequestUploadTypeId
}

#[deriving(Encodable)]
pub struct XMLHttpRequest {
    eventtarget: XMLHttpRequestEventTarget,
    ready_state: u16,
    timeout: u32,
    with_credentials: bool,
    upload: Option<JS<XMLHttpRequestUpload>>,
    response_url: DOMString,
    status: u16,
    status_text: ByteString,
    response_type: XMLHttpRequestResponseType,
    response_text: DOMString,
    response_xml: Option<JS<Document>>
}

impl XMLHttpRequest {
    pub fn new_inherited(owner: &JSRef<Window>) -> XMLHttpRequest {
        let mut xhr = XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestTypeId),
            ready_state: 0,
            timeout: 0u32,
            with_credentials: false,
            upload: None,
            response_url: "".to_owned(),
            status: 0,
            status_text: ByteString::new(vec!()),
            response_type: _empty,
            response_text: "".to_owned(),
            response_xml: None
        };
        xhr.upload.assign(Some(XMLHttpRequestUpload::new(owner)));
        xhr
    }
    pub fn new(window: &JSRef<Window>) -> Temporary<XMLHttpRequest> {
        reflect_dom_object(~XMLHttpRequest::new_inherited(window),
                           window,
                           XMLHttpRequestBinding::Wrap)
    }
    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Temporary<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(owner))
    }
}

pub trait XMLHttpRequestMethods {
    fn ReadyState(&self) -> u16;
    fn Open(&self, _method: ByteString, _url: DOMString);
    fn Open_(&self, _method: ByteString, _url: DOMString, _async: bool,
             _username: Option<DOMString>, _password: Option<DOMString>);
    fn SetRequestHeader(&self, _name: ByteString, _value: ByteString);
    fn Timeout(&self) -> u32;
    fn SetTimeout(&mut self, timeout: u32);
    fn WithCredentials(&self) -> bool;
    fn SetWithCredentials(&mut self, with_credentials: bool);
    fn Upload(&self) -> Temporary<XMLHttpRequestUpload>;
    fn Send(&self, _data: Option<DOMString>);
    fn Abort(&self);
    fn ResponseURL(&self) -> DOMString;
    fn Status(&self) -> u16;
    fn StatusText(&self) -> ByteString;
    fn GetResponseHeader(&self, _name: ByteString) -> Option<ByteString>;
    fn GetAllResponseHeaders(&self) -> ByteString;
    fn OverrideMimeType(&self, _mime: DOMString);
    fn ResponseType(&self) -> XMLHttpRequestResponseType;
    fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType);
    fn Response(&self, _cx: *JSContext) -> JSVal;
    fn ResponseText(&self) -> DOMString;
    fn GetResponseXML(&self) -> Option<Temporary<Document>>;
}

impl<'a> XMLHttpRequestMethods for JSRef<'a, XMLHttpRequest> {
    fn ReadyState(&self) -> u16 {
        self.ready_state
    }
    fn Open(&self, _method: ByteString, _url: DOMString) {

    }
    fn Open_(&self, _method: ByteString, _url: DOMString, _async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) {

    }
    fn SetRequestHeader(&self, _name: ByteString, _value: ByteString) {

    }
    fn Timeout(&self) -> u32 {
        self.timeout
    }
    fn SetTimeout(&mut self, timeout: u32) {
        self.timeout = timeout
    }
    fn WithCredentials(&self) -> bool {
        self.with_credentials
    }
    fn SetWithCredentials(&mut self, with_credentials: bool) {
        self.with_credentials = with_credentials
    }
    fn Upload(&self) -> Temporary<XMLHttpRequestUpload> {
        Temporary::new(self.upload.get_ref().clone())
    }
    fn Send(&self, _data: Option<DOMString>) {

    }
    fn Abort(&self) {

    }
    fn ResponseURL(&self) -> DOMString {
        self.response_url.clone()
    }
    fn Status(&self) -> u16 {
        self.status
    }
    fn StatusText(&self) -> ByteString {
        self.status_text.clone()
    }
    fn GetResponseHeader(&self, _name: ByteString) -> Option<ByteString> {
        None
    }
    fn GetAllResponseHeaders(&self) -> ByteString {
        ByteString::new(vec!())
    }
    fn OverrideMimeType(&self, _mime: DOMString) {

    }
    fn ResponseType(&self) -> XMLHttpRequestResponseType {
        self.response_type
    }
    fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType) {
        self.response_type = response_type
    }
    fn Response(&self, _cx: *JSContext) -> JSVal {
        NullValue()
    }
    fn ResponseText(&self) -> DOMString {
        self.response_text.clone()
    }
    fn GetResponseXML(&self) -> Option<Temporary<Document>> {
        self.response_xml.clone().map(|response| Temporary::new(response))
    }
}

impl Reflectable for XMLHttpRequest {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
    }
}

impl XMLHttpRequestDerived for EventTarget {
    fn is_xmlhttprequest(&self) -> bool {
        self.type_id == XMLHttpRequestTargetTypeId(XMLHttpRequestTypeId)
    }
}
