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
use dom::bindings::js::JS;
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
    upload: JS<XMLHttpRequestUpload>,
    response_url: DOMString,
    status: u16,
    status_text: ByteString,
    response_type: XMLHttpRequestResponseType,
    response_text: DOMString,
    response_xml: Option<JS<Document>>
}

impl XMLHttpRequest {
    pub fn new_inherited(owner: &JS<Window>) -> XMLHttpRequest {
        XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestTypeId),
            ready_state: 0,
            timeout: 0u32,
            with_credentials: false,
            upload: XMLHttpRequestUpload::new(owner),
            response_url: ~"",
            status: 0,
            status_text: ByteString::new(vec!()),
            response_type: _empty,
            response_text: ~"",
            response_xml: None
        }
    }
    pub fn new(window: &JS<Window>) -> JS<XMLHttpRequest> {
        reflect_dom_object(~XMLHttpRequest::new_inherited(window),
                           window,
                           XMLHttpRequestBinding::Wrap)
    }
    pub fn Constructor(owner: &JS<Window>) -> Fallible<JS<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(owner))
    }
    pub fn ReadyState(&self) -> u16 {
        self.ready_state
    }
    pub fn Open(&self, _method: ByteString, _url: DOMString) {

    }
    pub fn Open_(&self, _method: ByteString, _url: DOMString, _async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) {

    }
    pub fn SetRequestHeader(&self, _name: ByteString, _value: ByteString) {

    }
    pub fn Timeout(&self) -> u32 {
        self.timeout
    }
    pub fn SetTimeout(&mut self, timeout: u32) {
        self.timeout = timeout
    }
    pub fn WithCredentials(&self) -> bool {
        self.with_credentials
    }
    pub fn SetWithCredentials(&mut self, with_credentials: bool) {
        self.with_credentials = with_credentials
    }
    pub fn Upload(&self) -> JS<XMLHttpRequestUpload> {
        self.upload.clone()
    }
    pub fn Send(&self, _data: Option<DOMString>) {

    }
    pub fn Abort(&self) {

    }
    pub fn ResponseURL(&self) -> DOMString {
        self.response_url.clone()
    }
    pub fn Status(&self) -> u16 {
        self.status
    }
    pub fn StatusText(&self) -> ByteString {
        self.status_text.clone()
    }
    pub fn GetResponseHeader(&self, _name: ByteString) -> Option<ByteString> {
        None
    }
    pub fn GetAllResponseHeaders(&self) -> ByteString {
        ByteString::new(vec!())
    }
    pub fn OverrideMimeType(&self, _mime: DOMString) {

    }
    pub fn ResponseType(&self) -> XMLHttpRequestResponseType {
        self.response_type
    }
    pub fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType) {
        self.response_type = response_type
    }
    pub fn Response(&self, _cx: *JSContext) -> JSVal {
        NullValue()
    }
    pub fn ResponseText(&self) -> DOMString {
        self.response_text.clone()
    }
    pub fn GetResponseXML(&self) -> Option<JS<Document>> {
        self.response_xml.clone()
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
        match self.type_id {
            XMLHttpRequestTargetTypeId(XMLHttpRequestTypeId) => true,
            _ => false
        }
    }
}