/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/uievents/#interface-uievent
[Exposed=Window]
interface UIEvent : Event {
  [Throws] constructor(DOMString type, optional UIEventInit eventInitDict = {});
  //  readonly    attribute WindowProxy? view;
  readonly attribute Window? view;
    readonly    attribute long         detail;
};

// https://w3c.github.io/uievents/#dictdef-uieventinit-uieventinit
dictionary UIEventInit : EventInit {
  //  WindowProxy? view = null;
  Window? view = null;
    long         detail = 0;
};

// https://w3c.github.io/uievents/#idl-interface-UIEvent-initializers
partial interface UIEvent {
    // Deprecated in DOM Level 3
    undefined initUIEvent (
      DOMString typeArg,
      boolean bubblesArg,
      boolean cancelableArg,
      Window? viewArg,
      long detailArg
    );
};
