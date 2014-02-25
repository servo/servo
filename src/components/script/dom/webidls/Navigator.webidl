/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-navigator-object
 * http://www.w3.org/TR/tracking-dnt/
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-navigator-object
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator implements NavigatorID;
Navigator implements NavigatorLanguage;
Navigator implements NavigatorOnLine;
//Navigator implements NavigatorContentUtils;
//Navigator implements NavigatorStorageUtils;

[NoInterfaceObject]
interface NavigatorID {
  readonly attribute DOMString appName;
  [Throws]
  readonly attribute DOMString appVersion;
  [Throws]
  readonly attribute DOMString platform;
  [Throws]
  readonly attribute DOMString userAgent;

  // Spec has this as a const, but that's wrong because it should not
  // be on the interface object.
  //const DOMString product = "Gecko"; // for historical reasons
  readonly attribute DOMString product;
};

[NoInterfaceObject]
interface NavigatorLanguage {
  readonly attribute DOMString? language;
};

[NoInterfaceObject]
interface NavigatorOnLine {
  readonly attribute boolean onLine;
};

// http://www.w3.org/TR/tracking-dnt/ sort of
partial interface Navigator {
  readonly attribute DOMString doNotTrack;
};

// Mozilla-specific extensions
// nsIDOMNavigator
partial interface Navigator {
  // WebKit/Blink/Trident/Presto support this (hardcoded "Mozilla").
  [Throws]
  readonly attribute DOMString appCodeName;
  //[Throws]
  //readonly attribute DOMString oscpu;
  // WebKit/Blink support this; Trident/Presto do not.
  readonly attribute DOMString vendor;
  // WebKit/Blink supports this (hardcoded ""); Trident/Presto do not.
  readonly attribute DOMString vendorSub;
  // WebKit/Blink supports this (hardcoded "20030107"); Trident/Presto don't
  readonly attribute DOMString productSub;
  // WebKit/Blink/Trident/Presto support this.
  readonly attribute boolean cookieEnabled;
  [Throws]
  readonly attribute DOMString buildID;

  // WebKit/Blink/Trident/Presto support this.
  [Throws]
  boolean javaEnabled();
  // Everyone but WebKit/Blink supports this.  See bug 679971.
  boolean taintEnabled();
};
