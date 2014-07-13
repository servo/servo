/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::{Fallible, Security, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::EventTarget;

use servo_util::str::DOMString;
use servo_util::url::try_parse_url;

#[deriving(Encodable)]
pub struct Worker {
    eventtarget: EventTarget,
}

impl Worker {
    pub fn Constructor(global: &GlobalRef, scriptURL: DOMString) -> Fallible<Temporary<Worker>> {
        // Step 2-4.
        let _worker_url = match try_parse_url(scriptURL.as_slice(), Some(global.get_url())) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        Err(Security)
    }
}

pub trait WorkerMethods {
}

impl Reflectable for Worker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
