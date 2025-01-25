/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlsourceelement
[Exposed=Window]
interface HTMLSourceElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute DOMString src;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
            attribute DOMString srcset;
  [CEReactions]
            attribute DOMString sizes;
  [CEReactions]
            attribute DOMString media;
};
