/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::inheritance::Castable;
use servo_url::ServoUrl;

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::node::Node;
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::network_listener::ResourceTimingListener;

/// IframeContext is a wrapper around [`HTMLIFrameElement`] that implements the [`ResourceTimingListener`] trait.
/// Note: this implementation of `resource_timing_global` returns the parent document's global scope, not the iframe's global scope.
pub(crate) struct IframeContext<'a> {
    // The iframe element that this context is associated with.
    element: &'a HTMLIFrameElement,
    // The URL of the iframe document.
    url: ServoUrl,
}

impl<'a> IframeContext<'a> {
    /// Creates a new IframeContext from a reference to an HTMLIFrameElement.
    pub fn new(element: &'a HTMLIFrameElement) -> Self {
        Self {
            element,
            url: element.get_url(),
        }
    }
}

impl<'a> ResourceTimingListener for IframeContext<'a> {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName("iframe".to_string()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.element.upcast::<Node>().owner_doc().global()
    }
}
