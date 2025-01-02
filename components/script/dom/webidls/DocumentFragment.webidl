/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#interface-documentfragment
[Exposed=Window]
interface DocumentFragment : Node {
  [Throws] constructor();
};

DocumentFragment includes NonElementParentNode;
DocumentFragment includes ParentNode;
