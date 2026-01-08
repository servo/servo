/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::{HandleObject, HandleValue};
use net_traits::pub_domains::is_same_site;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::OriginBinding::OriginMethods;
use crate::dom::bindings::conversions::{
    ConversionResult, SafeFromJSValConvertible, StringificationBehavior, root_from_handlevalue,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlareaelement::HTMLAreaElement;
use crate::dom::html::htmlhyperlinkelementutils::{HyperlinkElement, HyperlinkElementTraits};
use crate::dom::url::URL;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

/// <https://html.spec.whatwg.org/multipage/#the-origin-interface>
#[dom_struct]
pub(crate) struct Origin {
    reflector: Reflector,
    #[no_trace]
    origin: ImmutableOrigin,
}

impl Origin {
    fn new_inherited(origin: ImmutableOrigin) -> Origin {
        Origin {
            reflector: Reflector::new(),
            origin,
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        origin: ImmutableOrigin,
        can_gc: CanGc,
    ) -> DomRoot<Origin> {
        reflect_dom_object_with_proto(
            Box::new(Origin::new_inherited(origin)),
            global,
            proto,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#extract-an-origin>
    fn extract_an_origin_from_platform_object(
        value: HandleValue,
        cx: JSContext,
        current_global: &GlobalScope,
    ) -> Option<ImmutableOrigin> {
        // <https://html.spec.whatwg.org/multipage/#the-origin-interface:extract-an-origin>
        if let Ok(origin_obj) = root_from_handlevalue::<Origin>(value, cx) {
            return Some(origin_obj.origin.clone());
        }

        // <https://url.spec.whatwg.org/#concept-url-origin>
        if let Ok(url_obj) = root_from_handlevalue::<URL>(value, cx) {
            return Some(url_obj.origin());
        }

        // <https://html.spec.whatwg.org/multipage/#window:extract-an-origin>
        if let Ok(window_obj) = root_from_handlevalue::<Window>(value, cx) {
            let window_origin = window_obj.origin();
            if !current_global.origin().same_origin_domain(window_origin) {
                return None;
            }
            return Some(window_origin.immutable().clone());
        }

        // <https://html.spec.whatwg.org/multipage/#api-for-a-and-area-elements:extract-an-origin>
        if let Ok(anchor_obj) = root_from_handlevalue::<HTMLAnchorElement>(value, cx) {
            anchor_obj.reinitialize_url();
            if let Some(ref url) = *anchor_obj.get_url().borrow() {
                return Some(url.origin());
            }
            return None;
        }

        // <https://html.spec.whatwg.org/multipage/#api-for-a-and-area-elements:extract-an-origin>
        if let Ok(area_obj) = root_from_handlevalue::<HTMLAreaElement>(value, cx) {
            area_obj.reinitialize_url();
            if let Some(ref url) = *area_obj.get_url().borrow() {
                return Some(url.origin());
            }
            return None;
        }

        None
    }
}

impl OriginMethods<crate::DomTypeHolder> for Origin {
    /// <https://html.spec.whatwg.org/multipage/#dom-origin-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Origin> {
        Origin::new(global, proto, ImmutableOrigin::new_opaque(), can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-origin-from>
    fn From(cx: JSContext, global: &GlobalScope, value: HandleValue) -> Fallible<DomRoot<Origin>> {
        let can_gc = CanGc::note();

        // Step 1. If value is a platform object:
        //   1. Let origin be the result of executing value's extract an origin operation.
        //   2. If origin is not null, then return a new Origin object whose origin is origin.
        if let Some(origin) = Origin::extract_an_origin_from_platform_object(value, cx, global) {
            return Ok(Origin::new(global, None, origin, can_gc));
        }

        // Step 2. If value is a string:
        if value.get().is_string() {
            let s = match DOMString::safe_from_jsval(
                cx,
                value,
                StringificationBehavior::Default,
                can_gc,
            ) {
                Ok(ConversionResult::Success(s)) => s,
                _ => return Err(Error::Type("Failed to convert value to string".to_string())),
            };

            // Step 2.1. Let parsedURL be the result of basic URL parsing value.
            // Step 2.2. If parsedURL is not failure, then return a new Origin object whose
            //           origin is set to parsedURL's origin.
            match ServoUrl::parse(&s.to_string()) {
                Ok(url) => return Ok(Origin::new(global, None, url.origin(), can_gc)),
                Err(_) => return Err(Error::Type("Failed to parse URL".to_string())),
            }
        }

        // Step 3. Throw a TypeError.
        Err(Error::Type(
            "Value must be a string or a platform object with an origin".to_string(),
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-origin-opaque>
    fn Opaque(&self) -> bool {
        !self.origin.is_tuple()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-origin-issameorigin>
    fn IsSameOrigin(&self, other: &Origin) -> bool {
        self.origin == other.origin
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-origin-issamesite>
    fn IsSameSite(&self, other: &Origin) -> bool {
        is_same_site(&self.origin, &other.origin)
    }
}
