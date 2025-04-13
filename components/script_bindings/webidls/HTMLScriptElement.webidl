/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlscriptelement
[Exposed=Window]
interface HTMLScriptElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, SetterThrows]
           attribute (TrustedScriptURL or USVString) src;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute boolean noModule;
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
  [CEReactions]
           attribute DOMString referrerPolicy;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLScriptElement-partial
partial interface HTMLScriptElement {
  [CEReactions]
           attribute DOMString event;
  [CEReactions]
           attribute DOMString htmlFor;
};
