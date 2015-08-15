/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API

callback BrowserElementNextPaintEventCallback = void ();

//enum BrowserFindCaseSensitivity { "case-sensitive", "case-insensitive" };
//enum BrowserFindDirection { "forward", "backward" };

//dictionary BrowserElementDownloadOptions {
//  DOMString? filename;
//  DOMString? referrer;
//};

//dictionary BrowserElementExecuteScriptOptions {
//  DOMString? url;
//  DOMString? origin;
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
  // CheckAnyPermissions="browser embed-widgets"]
  //void setVisible(boolean visible);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser embed-widgets"]
  //DOMRequest getVisible();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser embed-widgets"]
  //void setActive(boolean active);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser embed-widgets"]
  //boolean getActive();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser embed-widgets"]
  //void addNextPaintListener(BrowserElementNextPaintEventCallback listener);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser embed-widgets"]
  //void removeNextPaintListener(BrowserElementNextPaintEventCallback listener);
};

[NoInterfaceObject]
interface BrowserElementPrivileged {
  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //void sendMouseEvent(DOMString type,
  //                    unsigned long x,
  //                    unsigned long y,
  //                    unsigned long button,
  //                    unsigned long clickCount,
  //                    unsigned long modifiers);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // Func="TouchEvent::PrefEnabled",
  // CheckAnyPermissions="browser"]
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
   CheckAnyPermissions="browser"]
  void goBack();

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckAnyPermissions="browser"]
  void goForward();

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckAnyPermissions="browser"]
  void reload(optional boolean hardReload = false);

  [Throws,
   Pref="dom.mozBrowserFramesEnabled",
   CheckAnyPermissions="browser"]
  void stop();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //DOMRequest download(DOMString url,
  //                    optional BrowserElementDownloadOptions options);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //DOMRequest purgeHistory();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //DOMRequest getScreenshot([EnforceRange] unsigned long width,
  //                         [EnforceRange] unsigned long height,
  //                         optional DOMString mimeType="");

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //void zoom(float zoom);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //DOMRequest getCanGoBack();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //DOMRequest getCanGoForward();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //DOMRequest getContentDimensions();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAllPermissions="browser input-manage"]
  //DOMRequest setInputMethodActive(boolean isActive);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAllPermissions="browser nfc-manager"]
  //void setNFCFocus(boolean isFocus);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //void findAll(DOMString searchString, BrowserFindCaseSensitivity caseSensitivity);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //void findNext(BrowserFindDirection direction);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAnyPermissions="browser"]
  //void clearMatch();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // CheckAllPermissions="browser browser:universalxss"]
  //DOMRequest executeScript(DOMString script,
  //                         optional BrowserElementExecuteScriptOptions options);

};
