/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;

pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElement {
    pub fn Options(&self) -> @mut HTMLCollection {
        let window = self.htmlelement.element.node.owner_doc().document().window;
        HTMLCollection::new(window, ~[])
    }
}
