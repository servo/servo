/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

extern crate hyper;

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use hyper::header::Headers;
use hyper::header::{Accept, AcceptEncoding, Cookie, ContentLength, ContentType, Host};
use hyper::http::RawStatus;
use hyper::method::Method;
use protocol::JsonPacketStream;
use rustc_serialize::json;
use std::net::TcpStream;

struct HttpRequest {
    url: String,
    method: Method,
    headers: Headers,
    body: Option<Vec<u8>>,
}

struct HttpResponse {
    headers: Option<Headers>,
    status: Option<RawStatus>,
    body: Option<Vec<u8>>
}

pub struct NetworkEventActor {
    pub name: String,
    request: HttpRequest,
    response: HttpResponse,
}

#[derive(RustcEncodable)]
pub struct EventActor {
    pub actor: String,
    pub url: String,
    pub method: String,
    pub startedDateTime: String,
    pub isXHR: bool,
    pub private: bool
}

#[derive(RustcEncodable)]
pub struct ResponseCookiesMsg {
    pub cookies: u32,
}

#[derive(RustcEncodable)]
pub struct ResponseStartMsg {
    pub httpVersion: String,
    pub remoteAddress: String,
    pub remotePort: u32,
    pub status: String,
    pub statusText: String,
    pub headersSize: u32,
    pub discardResponseBody: bool,
}

#[derive(RustcEncodable)]
pub struct ResponseContentMsg {
    pub mimeType: String,
    pub contentSize: u32,
    pub transferredSize: u32,
    pub discardResponseBody: bool,
}


#[derive(RustcEncodable)]
pub struct ResponseHeadersMsg {
    pub headers: u32,
    pub headersSize: u32,
}


#[derive(RustcEncodable)]
pub struct RequestCookiesMsg {
    pub cookies: u32,
}

#[derive(RustcEncodable)]
struct GetRequestHeadersReply {
    from: String,
    headers: Vec<String>,
    headerSize: u8,
    rawHeaders: String
}

#[derive(RustcEncodable)]
struct GetResponseHeadersReply {
    from: String,
    headers: Vec<String>,
    headerSize: u8,
    rawHeaders: String
}

#[derive(RustcEncodable)]
struct GetResponseContentReply {
    from: String,
    content: Option<Vec<u8>>,
	contentDiscarded: bool,
}

#[derive(RustcEncodable)]
struct GetRequestPostDataReply {
    from: String,
    postData: Option<Vec<u8>>,
    postDataDiscarded: bool
}

#[derive(RustcEncodable)]
struct GetRequestCookiesReply {
    from: String,
    cookies: Vec<u8>
}

#[derive(RustcEncodable)]
struct GetResponseCookiesReply {
    from: String,
    cookies: Vec<u8>
}

#[derive(RustcEncodable)]
struct Timings {
    blocked: u32,
	dns: u32,
	connect: u32,
	send: u32,
	wait: u32,
	receive: u32,
}

#[derive(RustcEncodable)]
struct GetEventTimingsReply {
    from: String,
    timings: Timings,
	totalTime: u32,
}

#[derive(RustcEncodable)]
struct GetSecurityInfoReply {
    from: String,
    seuritInfo: String,
}


impl Actor for NetworkEventActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getRequestHeaders" => {
                // TODO: Pass the correct values for headers, headerSize, rawHeaders
                let mut headersIter = self.request.headers.iter();
				let headersSize = self.request.headers.len() as u8;
				let mut headerNames = Vec::new();				
				let mut rawHeadersString = "".to_owned();
				for i in 0..headersSize {
					//let item = headersIter.next();
					
					if let Some(item) = headersIter.next() {
                		let name = item.name();
						let value = item.value_string();
						headerNames.push(name.to_owned());
						rawHeadersString = rawHeadersString + name + ":" + &value+"\r\n";
					}
		
				}
                
                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers: headerNames,
                    headerSize: headersSize,
                    rawHeaders: rawHeadersString,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getRequestCookies" => {
					let mut newVec = Vec::new();
					if let Some(req_cookies) = self.request.headers.get_raw("Cookie") {
        				for cookie in req_cookies.iter() {
            			if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                			newVec= cookie_value.into_bytes();
            				}
        				}
    				}

					let msg = GetRequestCookiesReply {
                    from: self.name(),
                    cookies: newVec,
               		 };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getRequestPostData" => {
				
				if let Some(list) = self.request.body.clone() {
					//let mut newVec = list.clone();
                	let msg = GetRequestPostDataReply {
                    	from: self.name(),
    					postData: Some(list),
    					postDataDiscarded: false,
                	};
				stream.write_json_packet(&msg);	
				}else{
					let msg = GetRequestPostDataReply {
                    	from: self.name(),
    					postData: None,
    					postDataDiscarded: false,
                	};
				stream.write_json_packet(&msg);
				}
				
                ActorMessageStatus::Processed
            }
            "getResponseHeaders" => {
				if let Some(res_headers) = self.response.headers.clone() {
		            let mut headersIter = res_headers.iter();
					let headersSize = res_headers.len() as u8;
					let mut headerNames = Vec::new();				
					let mut rawHeadersString = "".to_owned();
					for i in 0..headersSize {
						if let Some(item) = headersIter.next() {
		            		let name = item.name();
							let value = item.value_string();
							headerNames.push(name.to_owned());
							rawHeadersString = rawHeadersString + name + ":" + &value+"\r\n";
						}
		
					}
		            
		            let msg = GetResponseHeadersReply {
		                from: self.name(),
		                headers: headerNames,
		                headerSize: headersSize,
		                rawHeaders: rawHeadersString,
		            };
		            stream.write_json_packet(&msg);
					}
		            ActorMessageStatus::Processed
            }
            "getResponseCookies" => {
                let mut newVec = Vec::new();
					if let Some(res_cookies) = self.request.headers.get_raw("set-cookie") {
        				for cookie in res_cookies.iter() {
            			if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                			newVec= cookie_value.into_bytes();
            				}
        				}
    				}

					let msg = GetResponseCookiesReply {
                    from: self.name(),
                    cookies: newVec,
               		 };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getResponseContent" => {
                if let Some(list) = self.response.body.clone() {
                	let msg = GetResponseContentReply {
                    	from: self.name(),
    					content: Some(list),
    					contentDiscarded: false,
                	};
				stream.write_json_packet(&msg);	
				}else{
					let msg = GetResponseContentReply {
                    	from: self.name(),
    					content: None,
    					contentDiscarded: false,
                	};
				stream.write_json_packet(&msg);
				}
				
                ActorMessageStatus::Processed
            }
			"getEventTimings" => {
				// TODO: This is a fake timings msg
				let timingsObj = Timings {
									blocked: 0,
									dns: 0,
									connect: 0,
									send: 0,
									wait: 0,
									receive: 0,
								};
				// TODO: Send the correct values for all these fields. 
				let msg = GetEventTimingsReply {
    						 from: self.name(),
   							 timings: timingsObj,
							 totalTime: 0,
						   };
				stream.write_json_packet(&msg);
				ActorMessageStatus::Processed
			}
			"getSecurityInfo" => {
				// TODO: Send the correct values for securityInfo.
				let msg = GetSecurityInfoReply {
    						from: self.name(),
    						seuritInfo: "".to_owned(),
						  };
				stream.write_json_packet(&msg);
				ActorMessageStatus::Processed
			}
            _ => ActorMessageStatus::Ignored
        })
    }
}

impl NetworkEventActor {
    pub fn new(name: String) -> NetworkEventActor {
        NetworkEventActor {
            name: name,
            request: HttpRequest {
                url: String::new(),
                method: Method::Get,
                headers: Headers::new(),
                body: None
            },
            response: HttpResponse {
                headers: None,
                status: None,
                body: None,
            }
        }
    }

    pub fn add_request(&mut self, request: DevtoolsHttpRequest) {
        self.request.url = request.url.serialize();
        self.request.method = request.method.clone();
        self.request.headers = request.headers.clone();
        self.request.body = request.body;
    }

    pub fn add_response(&mut self, response: DevtoolsHttpResponse) {
        self.response.headers = response.headers.clone();
        self.response.status = response.status.clone();
        self.response.body = response.body.clone();
     }

    pub fn event_actor(&self) -> EventActor {
        // TODO: Send the correct values for startedDateTime, isXHR, private
        EventActor {
            actor: self.name(),
            url: self.request.url.clone(),
            method: format!("{}", self.request.method),
            startedDateTime: "2015-04-22T20:47:08.545Z".to_owned(),
            isXHR: false,
            private: false,
        }
    }

    pub fn response_start(&self) -> ResponseStartMsg {
        // TODO: Send the correct values for all these fields.
        //       This is a fake message.

		//let new_headers = self.response.headers.clone();
		let mut hSize=0;
		if let Some(res_headers) = self.response.headers.clone() {
			hSize = res_headers.len() as u32;
		}
		
		let mut status_code=0;
		let mut status_message="".to_owned();
		if let Some(res_status) = self.response.status.clone() {
			let RawStatus(code,text_Res) = res_status;
			status_code = code;
			status_message = text_Res.into_owned();
		}
        ResponseStartMsg {
            httpVersion: "HTTP/1.1".to_owned(),
            remoteAddress: "63.245.217.43".to_owned(),
            remotePort: 443,
            status: status_code.to_string(),
            statusText: status_message,
            headersSize: hSize,
            discardResponseBody: false
        }
    }
	
	pub fn response_content(&self) -> ResponseContentMsg {
				let mut mString = "".to_owned();
				if let Some(list) = self.response.headers.clone() {
            		let mimetype = match list.get() {
						Some(&ContentType(ref mime)) =>  Some(mime),
						None => None
					};
				if let Some(mtype)= mimetype {
					mString = mtype.to_string();

				}				
				}
			// TODO: Set correct values when response's body is sent to the devtools in http_loader.
		ResponseContentMsg {
			mimeType: mString,
			contentSize: 0,
			transferredSize: 0,
			discardResponseBody: false,
		}
	}

	pub fn response_cookies(&self) -> ResponseCookiesMsg {
		
		let mut cookies_size =0;		
		if let Some(list) = self.response.headers.clone() {
    		let cookietype = match list.get() {
				Some(&Cookie(ref cookie)) =>  Some(cookie),
				None => None
			};
			if let Some(cookie)= cookietype {
						cookies_size  = cookie.len();

			}
		
		}

		ResponseCookiesMsg {
			cookies: cookies_size as u32,
		}
	}

	pub fn response_headers(&self) -> ResponseHeadersMsg {
		
		let mut headers_size=0;
		if let Some(res_headers) = self.response.headers.clone() {
				headers_size = res_headers.len() as u32;
				
		}
		// TODO: Set correct value for headersSize.
		ResponseHeadersMsg {
			headers: headers_size,
			headersSize: headers_size,
		}

	}

	pub fn request_cookies(&self) -> RequestCookiesMsg {
		
		let mut cookies_size =0;		
		 let list = self.request.headers.clone(); 
    		let cookietype = match list.get() {
				Some(&Cookie(ref cookie)) =>  Some(cookie),
				None => None
			};
			if let Some(cookie)= cookietype {
						cookies_size  = cookie.len();

			}
		
		

		RequestCookiesMsg {
			cookies: cookies_size as u32,
		}
	}

}
