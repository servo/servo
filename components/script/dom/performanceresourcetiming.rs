/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::PerformanceResourceTimingMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom_struct::dom_struct;
use servo_url::ServoUrl;


#[dom_struct]
pub struct PerformanceResourceTiming {
    reflector_: Reflector,
    navigation_start: u64,
    navigation_start_precise: u64,
    // returns the resolved URL of the requested resource
    // does not change even if redirected to a different URl
    name: ServoUrl,
}

impl PerformanceResourceTiming {
    // TODO
}

// https://w3c.github.io/resource-timing/
impl PerformanceResourceTimingMethods for PerformanceResourceTiming {
    // https://w3c.github.io/resource-timing/
    fn InitiatorType(&self) -> DOMString {
        // TODO
        Default::default()
    }

    // https://w3c.github.io/resource-timing/
    fn NextHopProtocol(&self) -> DOMString {
        // TODO
        Default::default()
    }

    // https://w3c.github.io/resource-timing/
    fn WorkerStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn RedirectStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn RedirectEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn FetchStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn DomainLookupStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn DomainLookupEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ConnectStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ConnectEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn SecureConnectionStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn RequestStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ResponseStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ResponseEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

}
