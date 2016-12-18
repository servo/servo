/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom-view/#dom-mediaquerylistevent-mediaquerylistevent
[Constructor(DOMString type, optional MediaQueryListEventInit eventInitDict), Exposed=(Window)]
interface MediaQueryListEvent : Event {
  readonly attribute DOMString media;
  readonly attribute boolean matches;
};

dictionary MediaQueryListEventInit : EventInit {
  DOMString media = "";
  boolean matches = false;
};
