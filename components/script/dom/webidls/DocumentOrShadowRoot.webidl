/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#documentorshadowroot
 * https://w3c.github.io/webcomponents/spec/shadow/#extensions-to-the-documentorshadowroot-mixin
 */

interface mixin DocumentOrShadowRoot {
  // Selection? getSelection();
  Element? elementFromPoint (double x, double y);
  sequence<Element> elementsFromPoint (double x, double y);
  // CaretPosition? caretPositionFromPoint (double x, double y);
  readonly attribute Element? activeElement;
  readonly attribute StyleSheetList styleSheets;
};
