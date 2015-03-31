/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#navigator
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator implements NavigatorID;
//Navigator implements NavigatorLanguage;
//Navigator implements NavigatorOnLine;
//Navigator implements NavigatorContentUtils;
//Navigator implements NavigatorStorageUtils;
//Navigator implements NavigatorPlugins;

// http://www.whatwg.org/html/#navigatorid
[NoInterfaceObject/*, Exposed=Window,Worker*/]
interface NavigatorID {
  readonly attribute DOMString appCodeName; // constant "Mozilla"
  readonly attribute DOMString appName;
  readonly attribute DOMString appVersion;
  readonly attribute DOMString platform;
  readonly attribute DOMString product; // constant "Gecko"
  boolean taintEnabled(); // constant false
  readonly attribute DOMString userAgent;
};
