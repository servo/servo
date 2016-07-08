/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlscriptelement
[Exposed=(Window,Worker)]
interface HTMLScriptElement : HTMLElement {
           attribute DOMString src;
           attribute DOMString type;
           attribute DOMString charset;
  //         attribute boolean async;
           attribute boolean defer;
  //         attribute DOMString crossOrigin;
           [Pure]
           attribute DOMString text;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLScriptElement-partial
partial interface HTMLScriptElement {
           attribute DOMString event;
           attribute DOMString htmlFor;
};
