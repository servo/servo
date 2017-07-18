/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlscriptelement
[HTMLConstructor]
interface HTMLScriptElement : HTMLElement {
  [CEReactions]
           attribute DOMString src;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute DOMString charset;
  [CEReactions]
           attribute boolean async;
  [CEReactions]
           attribute boolean defer;
  [CEReactions]
           attribute DOMString? crossOrigin;
  [CEReactions, Pure]
           attribute DOMString text;
  [CEReactions]
           attribute DOMString integrity;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLScriptElement-partial
partial interface HTMLScriptElement {
  [CEReactions]
           attribute DOMString event;
  [CEReactions]
           attribute DOMString htmlFor;
};
