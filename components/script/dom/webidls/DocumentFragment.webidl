/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#interface-documentfragment
[Constructor, Exposed=(Window,Worker)]
interface DocumentFragment : Node {
};

DocumentFragment implements NonElementParentNode;
DocumentFragment implements ParentNode;
