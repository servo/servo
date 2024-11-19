/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/#htmlorsvgelement
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface mixin HTMLOrSVGElement {
  // [SameObject] readonly attribute DOMStringMap dataset;
  // attribute DOMString nonce; // intentionally no [CEReactions]

  [CEReactions] attribute boolean autofocus;
  // [CEReactions] attribute long tabIndex;
  // undefined focus(optional FocusOptions options = {});
  // undefined blur();
};
