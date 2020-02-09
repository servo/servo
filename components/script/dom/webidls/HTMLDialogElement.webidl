/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmldialogelement
[Exposed=Window]
interface HTMLDialogElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
  attribute boolean open;
  attribute DOMString returnValue;
  // [CEReactions]
  // void show();
  // [CEReactions]
  // void showModal();
  [CEReactions]
  void close(optional DOMString returnValue);
};
