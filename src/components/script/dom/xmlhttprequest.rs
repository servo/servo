/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use collections::hashmap::HashMap;
use dom::bindings::codegen::BindingDeclarations::XMLHttpRequestBinding;
use dom::bindings::str::ByteString;
use self::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use self::XMLHttpRequestBinding::XMLHttpRequestResponseTypeValues::_empty;
use dom::bindings::codegen::InheritTypes::XMLHttpRequestDerived;
use dom::bindings::error::{ErrorResult, InvalidState, Network, Syntax, Security};
use dom::document::Document;
use dom::eventtarget::{EventTarget, XMLHttpRequestTargetTypeId};
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable};
use dom::bindings::trace::Untraceable;
use js::jsapi::JSContext;
use js::jsval::JSVal;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;
use net::resource_task::{load_whole_resource};
use servo_util::str::DOMString;
use servo_util::url::{parse_url, try_parse_url};
use url::Url;



// As send() start accepting more and more parameter types,
// change this to the appropriate type from UnionTypes, eg
// use SendParam = dom::bindings::codegen::UnionTypes::StringOrFormData;
type SendParam = DOMString;

#[deriving(Eq,Encodable)]
pub enum XMLHttpRequestId {
    XMLHttpRequestTypeId,
    XMLHttpRequestUploadTypeId
}

#[deriving(Eq, Encodable)]
enum XMLHttpRequestState {
    Unsent = 0u16,
    Opened = 1u16,
    HeadersReceived = 2u16,
    Loading = 3u16,
    Done = 4u16,
}

#[deriving(Encodable)]
pub struct XMLHttpRequest {
    eventtarget: XMLHttpRequestEventTarget,
    ready_state: XMLHttpRequestState,
    timeout: u32,
    with_credentials: bool,
    upload: Option<JS<XMLHttpRequestUpload>>,
    response_url: DOMString,
    status: u16,
    status_text: ByteString,
    response: ByteString,
    response_type: XMLHttpRequestResponseType,
    response_text: DOMString,
    response_xml: Option<JS<Document>>,

    // Associated concepts
    request_method: ByteString,
    request_url: Untraceable<Url>,
    request_headers: HashMap<ByteString, ByteString>,
    request_body: SendParam,
    sync: bool,
    upload_complete: bool,
    upload_events: bool,
    send_flag: bool,

    global: JS<Window>
}

impl XMLHttpRequest {
    pub fn new_inherited(owner: &JSRef<Window>) -> XMLHttpRequest {
        let mut xhr = XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestTypeId),
            ready_state: Unsent,
            timeout: 0u32,
            with_credentials: false,
            upload: None,
            response_url: "".to_owned(),
            status: 0,
            status_text: ByteString::new(vec!()),
            response: ByteString::new(vec!()),
            response_type: _empty,
            response_text: "".to_owned(),
            response_xml: None,

            request_method: ByteString::new(vec!()),
            request_url: Untraceable::new(parse_url("", None)),
            request_headers: HashMap::new(),
            request_body: "".to_owned(),
            sync: false,
            send_flag: false,

            upload_complete: false,
            upload_events: false,

            global: owner.unrooted()
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
    fn Open(&mut self, _method: ByteString, _url: DOMString) -> ErrorResult;
    fn Open_(&mut self, _method: ByteString, _url: DOMString, _async: bool,
             _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult;
    fn SetRequestHeader(&self, _name: ByteString, _value: ByteString);
    fn Timeout(&self) -> u32;
    fn SetTimeout(&mut self, timeout: u32);
    fn WithCredentials(&self) -> bool;
    fn SetWithCredentials(&mut self, with_credentials: bool);
    fn Upload(&self) -> Temporary<XMLHttpRequestUpload>;
    fn Send(&mut self, _data: Option<SendParam>) -> ErrorResult;
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
        self.ready_state as u16
    }
    fn Open(&mut self, method: ByteString, url: DOMString) -> ErrorResult {
        self.request_method = method;
        self.sync = true; //XXXManishearth the default should be changed later

        // Step 2
        let base: Option<Url> = Some(self.global.root().get_url());
        match self.request_method.to_lower().as_str() {
            Some("get") => {

                // Step 5
                self.request_method = self.request_method.to_lower();

                // Step 6
                let parsed_url = match try_parse_url(url, base) {
                    Ok(parsed) => parsed,
                    Err(_) => return Err(Syntax) // Step 7
                };
                // XXXManishearth Do some handling of username/passwords, and abort existing requests

                // Step 12
                self.request_url = Untraceable::new(parsed_url);
                self.request_headers = HashMap::new();
                self.send_flag = false;
                // XXXManishearth Set response to a NetworkError

                // Step 13
                self.ready_state = Opened;
                //XXXManishearth fire a progressevent
                Ok(())
            },
            // XXXManishearth Handle other standard methods
            Some("connect") | Some("trace") | Some("track") => {
                Err(Security) // Step 4
            },
            None => Err(Syntax),
            _ => {
                if self.request_method.is_token() {
                    Ok(()) // XXXManishearth handle extension methods
                } else {
                    Err(Syntax) // Step 3
                }
            }
        }
    }
    fn Open_(&mut self, _method: ByteString, _url: DOMString, _async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult {
        Ok(())
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
    fn Send(&mut self, data: Option<DOMString>) -> ErrorResult {
        // XXXManishearth handle async requests, POSTdata, and headers
        if self.ready_state != Opened || self.send_flag {
            return Err(InvalidState); // Step 1, 2
        }

        let data = match self.request_method.to_lower().as_str() {
            Some("get") | Some("head") => None, // Step 3
            _ => data
        };

        // Step 6
        self.upload_complete = false;
        self.upload_events = false;
        // XXXManishearth handle upload events

        // Step 9
        self.send_flag = true;

        let resource_task = self.global.root().page().resource_task.deref().clone();

        // Step 10, 13
        let fetched = load_whole_resource(&resource_task, self.request_url.clone());
        self.ready_state = Done;

        // Error and result handling
        match fetched {
            Ok((_, text)) => {
                self.response = ByteString::new(text)
            },
            Err(_) => {
                self.send_flag = false;
                // XXXManishearth set response to NetworkError
                if !self.upload_complete {
                    self.upload_complete = true;
                    // XXXManishearth handle upload progress
                }
                // XXXManishearth fire some progress events
                return Err(Network)
            }
        };
        Ok(())
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
    fn Response(&self, cx: *JSContext) -> JSVal {
        self.response.to_jsval(cx)
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
        match self.type_id {
            XMLHttpRequestTargetTypeId(XMLHttpRequestTypeId) => true,
            _ => false
        }
    }
}