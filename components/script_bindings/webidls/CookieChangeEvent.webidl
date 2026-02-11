/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://cookiestore.spec.whatwg.org/

[Exposed=Window,
 SecureContext]
interface CookieChangeEvent : Event {
  constructor(DOMString type, optional CookieChangeEventInit eventInitDict = {});
  /*[SameObject]*/ readonly attribute /*FrozenArray<CookieListItem>*/ any changed;
  /*[SameObject]*/ readonly attribute /*FrozenArray<CookieListItem>*/ any deleted;
};

dictionary CookieChangeEventInit : EventInit {
  CookieList changed;
  CookieList deleted;
};