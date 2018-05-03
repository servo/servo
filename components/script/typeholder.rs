/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::trace::JSTraceable;
use dom::domparser::DOMParserTrait;
use dom::servoparser::{ServoParser, TokenizerTrait};
use dom::xmlhttprequest::XHRTimeoutCallbackTrait;
use dom::xmlhttprequest::XMLHttpRequestTrait;
use malloc_size_of::MallocSizeOf;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::marker::Copy;
use std::marker::Send;
use std::marker::Sized;


pub trait TypeHolderTrait:
    MallocSizeOf +
    JSTraceable +
    Debug +
    Default +
    Send  +
    Clone +
    Copy +
    PartialEq +
    'static
{
    type ServoParser: ServoParser<Self>;
    type HtmlTokenizer: TokenizerTrait<Self>;
    type AsyncHtmlTokenizer: TokenizerTrait<Self>;
    type XmlTokenizer: TokenizerTrait<Self>;
    type DOMParser: DOMParserTrait<Self>;
    type XMLHttpRequest: XMLHttpRequestTrait<Self>;
    type XHRTimeoutCallback: XHRTimeoutCallbackTrait<Self>;
}

