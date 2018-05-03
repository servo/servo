use dom::servoparser::{ServoParser, TokenizerTrait};
use dom::bindings::trace::JSTraceable;
use malloc_size_of::MallocSizeOf;
use std::marker::Sized;
use std::marker::Send;
use dom::bindings::inheritance::Castable;
use std::clone::Clone;
use std::marker::Copy;
use std::fmt::Debug;
use std::cmp::PartialEq;
use dom::domparser::DOMParserTrait;
use dom::xmlhttprequest::XMLHttpRequestTrait;
use dom::xmlhttprequest::XHRTimeoutCallbackTrait;


pub trait TypeHolderTrait: MallocSizeOf + JSTraceable  + 'static + Sized + Default + Send  + Clone + Copy + Debug + PartialEq {
    type ServoParser: ServoParser<Self>;
    type HtmlTokenizer: TokenizerTrait<Self>;
    type AsyncHtmlTokenizer: TokenizerTrait<Self>;
    type XmlTokenizer: TokenizerTrait<Self>;
    type DOMParser: DOMParserTrait<Self>;
    type XMLHttpRequest: XMLHttpRequestTrait<Self>;
    type XHRTimeoutCallback: XHRTimeoutCallbackTrait<Self>;
}