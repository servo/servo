/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-script-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 */

interface HTMLScriptElement : HTMLElement {
  [SetterThrows]
  attribute DOMString src;
  [SetterThrows]
  attribute DOMString type;
  [SetterThrows]
  attribute DOMString charset;
  [SetterThrows]
  attribute boolean async;
  [SetterThrows]
  attribute boolean defer;
  [SetterThrows]
  attribute DOMString crossOrigin;
  [SetterThrows]
  attribute DOMString text;
};

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLScriptElement {
  [SetterThrows]
  attribute DOMString event;
  [SetterThrows]
  attribute DOMString htmlFor;
};

