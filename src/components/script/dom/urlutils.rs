/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::Fallible;
use servo_util::str::DOMString;

pub trait URLUtils {
    fn Href(&self) -> DOMString {
        ~""
    }

    fn SetHref(&self, _href: DOMString) -> Fallible<()> {
        Ok(())
    }

    fn Origin(&self) -> DOMString {
        ~""
    }

    fn Protocol(&self) -> DOMString {
        ~""
    }

    fn SetProtocol(&self, _protocol: DOMString) {
    }

    fn Username(&self) -> DOMString {
        ~""
    }

    fn SetUsername(&self, _username: DOMString) {
    }

    fn Password(&self) -> DOMString {
        ~""
    }

    fn SetPassword(&self, _password: DOMString) {
    }

    fn Host(&self) -> DOMString {
        ~""
    }

    fn SetHost(&self, _host: DOMString) {
    }

    fn Hostname(&self) -> DOMString {
        ~""
    }

    fn SetHostname(&self, _hostname: DOMString) {
    }

    fn Port(&self) -> DOMString {
        ~""
    }

    fn SetPort(&self, _port: DOMString) {
    }

    fn Pathname(&self) -> DOMString {
        ~""
    }

    fn SetPathname(&self, _pathname: DOMString) {
    }

    fn Search(&self) -> DOMString {
        ~""
    }

    fn SetSearch(&self, _search: DOMString) {
    }

    fn Hash(&self) -> DOMString {
        ~""
    }

    fn SetHash(&self, _hash: DOMString) {
    }
}
