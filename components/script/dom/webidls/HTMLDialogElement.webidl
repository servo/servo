/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmldialogelement
[HTMLConstructor]
interface HTMLDialogElement : HTMLElement {
  attribute boolean open;
  attribute DOMString returnValue;
  //void show(optional (MouseEvent or Element) anchor);
  //void showModal(optional (MouseEvent or Element) anchor);
  void close(optional DOMString returnValue);
};
