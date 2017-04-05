/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// enum ScrollRestoration { "auto", "manual" };

// https://html.spec.whatwg.org/multipage/#the-history-interface
[Exposed=(Window,Worker)]
interface History {
  readonly attribute unsigned long length;
  // attribute ScrollRestoration scrollRestoration;
  // readonly attribute any state;
  [Throws] void go(optional long delta = 0);
  void back();
  void forward();
  // void pushState(any data, DOMString title, optional USVString? url = null);
  // void replaceState(any data, DOMString title, optional USVString? url = null);
};
