/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DissimilarOriginLocationBinding::DissimilarOriginLocationMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::dissimilaroriginwindow::DissimilarOriginWindow;
use crate::script_runtime::CanGc;

/// Represents a dissimilar-origin `Location` that exists in another script thread.
///
/// Since the `Location` is in a different script thread, we cannot access it
/// directly, but some of its accessors (for example setting `location.href`)
/// still need to function.

#[dom_struct]
pub(crate) struct DissimilarOriginLocation {
    /// The reflector. Once we have XOWs, this will have a cross-origin
    /// wrapper placed around it.
    reflector: Reflector,

    /// The window associated with this location.
    window: Dom<DissimilarOriginWindow>,
}

impl DissimilarOriginLocation {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(window: &DissimilarOriginWindow) -> DissimilarOriginLocation {
        DissimilarOriginLocation {
            reflector: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub(crate) fn new(window: &DissimilarOriginWindow) -> DomRoot<DissimilarOriginLocation> {
        reflect_dom_object(
            Box::new(DissimilarOriginLocation::new_inherited(window)),
            window,
            CanGc::note(),
        )
    }
}

impl DissimilarOriginLocationMethods<crate::DomTypeHolder> for DissimilarOriginLocation {
    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn GetHref(&self) -> Fallible<USVString> {
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn SetHref(&self, _: USVString) -> ErrorResult {
        // TODO: setting href on a cross-origin window should succeed?
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, _: USVString) -> Fallible<()> {
        // TODO: setting href on a cross-origin window should succeed?
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-replace
    fn Replace(&self, _: USVString) -> Fallible<()> {
        // TODO: replacing href on a cross-origin window should succeed?
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    fn Reload(&self) -> Fallible<()> {
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Stringifier(&self) -> Fallible<DOMString> {
        Err(Error::Security)
    }
}
