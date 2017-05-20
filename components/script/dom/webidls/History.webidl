/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// enum ScrollRestoration { "auto", "manual" };

// https://html.spec.whatwg.org/multipage/#the-history-interface
[Exposed=(Window,Worker)]
interface History {
  [Throws]
  readonly attribute unsigned long length;
  // [Throws]
  // attribute ScrollRestoration scrollRestoration;
  // [Throws]
  // readonly attribute any state;
  [Throws]
  void go(optional long delta = 0);
  [Throws]
  void back();
  [Throws]
  void forward();
  // [Throws]
  // void pushState(any data, DOMString title, optional USVString? url = null);
  // [Throws]
  // void replaceState(any data, DOMString title, optional USVString? url = null);
};
