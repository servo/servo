/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API

callback BrowserElementNextPaintEventCallback = void ();

//dictionary BrowserElementDownloadOptions {
//  DOMString? filename;
//};

[NoInterfaceObject]
interface BrowserElement {
};

BrowserElement implements BrowserElementCommon;
BrowserElement implements BrowserElementPrivileged;

[NoInterfaceObject]
interface BrowserElementCommon {
  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser embed-widgets"]
  //void setVisible(boolean visible);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser embed-widgets"]
  //DOMRequest getVisible();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser embed-widgets"]
  //void addNextPaintListener(BrowserElementNextPaintEventCallback listener);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser embed-widgets"]
  //void removeNextPaintListener(BrowserElementNextPaintEventCallback listener);
};

[NoInterfaceObject]
interface BrowserElementPrivileged {
  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //void sendMouseEvent(DOMString type,
  //                    unsigned long x,
  //                    unsigned long y,
  //                    unsigned long button,
  //                    unsigned long clickCount,
  //                    unsigned long modifiers);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // Func="TouchEvent::PrefEnabled",
  // CheckPermissions="browser"]
  //void sendTouchEvent(DOMString type,
  //                    sequence<unsigned long> identifiers,
  //                    sequence<long> x,
  //                    sequence<long> y,
  //                    sequence<unsigned long> rx,
  //                    sequence<unsigned long> ry,
  //                    sequence<float> rotationAngles,
  //                    sequence<float> forces,
  //                    unsigned long count,
  //                    unsigned long modifiers);

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckPermissions="browser"]
  void goBack();

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckPermissions="browser"]
  void goForward();

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckPermissions="browser"]
  void reload(optional boolean hardReload = false);

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckPermissions="browser"]
  void stop();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest download(DOMString url,
  //                    optional BrowserElementDownloadOptions options);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest purgeHistory();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest getScreenshot([EnforceRange] unsigned long width,
  //                         [EnforceRange] unsigned long height,
  //                         optional DOMString mimeType="");

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //void zoom(float zoom);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest getCanGoBack();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest getCanGoForward();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest getContentDimensions();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //DOMRequest setInputMethodActive(boolean isActive);

  // Additional |nfc-manager| permission is required for setNFCFocus API
  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckPermissions="browser"]
  //void setNFCFocus(boolean isFocus);
};
