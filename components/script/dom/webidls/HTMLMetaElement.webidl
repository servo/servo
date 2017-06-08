/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlmetaelement
[HTMLConstructor]
interface HTMLMetaElement : HTMLElement {
             attribute DOMString name;
  //         attribute DOMString httpEquiv;
             attribute DOMString content;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLMetaElement-partial
partial interface HTMLMetaElement {
  //         attribute DOMString scheme;
};
