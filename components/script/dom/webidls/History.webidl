/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/#the-history-interface
enum ScrollRestoration { "auto", "manual" };

interface History {
  // readonly attribute unsigned long length;
  // attribute ScrollRestoration scrollRestoration;
  // readonly attribute any state;
  // void go(optional long delta = 0);
  void back();
  void forward();
  // void pushState(any data, DOMString title, optional DOMString? url = null);
  // void replaceState(any data, DOMString title, optional DOMString? url = null);
};
