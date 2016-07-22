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

[NoInterfaceObject, Exposed=(Window,Worker)]
interface BrowserElement {
};

dictionary BrowserElementSecurityChangeDetail {

  // state:
  //   "insecure" indicates that the data corresponding to
  //     the request was received over an insecure channel.
  //
  //   "broken" indicates an unknown security state.  This
  //     may mean that the request is being loaded as part
  //     of a page in which some content was received over
  //     an insecure channel.
  //
  //   "secure" indicates that the data corresponding to the
  //     request was received over a secure channel.
  DOMString state;

  // trackingState:
  //   "loaded_tracking_content": tracking content has been loaded.
  //   "blocked_tracking_content": tracking content has been blocked from loading.
  DOMString trackingState;

  // mixedState:
  //   "blocked_mixed_active_content": Mixed active content has been blocked from loading.
  //   "loaded_mixed_active_content": Mixed active content has been loaded.
  DOMString mixedState;

  boolean extendedValidation;
  boolean trackingContent;
  boolean mixedContent;
};

dictionary BrowserElementErrorEventDetail {
  // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsererror
  // just requires a "type" field, but we also provide
  // an optional human-readable description, and
  // an optional machine-readable report (e.g. a backtrace for panics)
  DOMString type;
  DOMString description;
  DOMString report;
  DOMString version;
};

dictionary BrowserElementLocationChangeEventDetail {
  DOMString url;
  boolean canGoBack;
  boolean canGoForward;
};

dictionary BrowserElementIconChangeEventDetail {
  DOMString rel;
  DOMString href;
  DOMString sizes;
};

dictionary BrowserShowModalPromptEventDetail {
  DOMString promptType;
  DOMString title;
  DOMString message;
  DOMString returnValue;
  // TODO(simartin) unblock() callback
};

dictionary BrowserElementOpenTabEventDetail {
  // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowseropentab
  DOMString url;
};

dictionary BrowserElementOpenWindowEventDetail {
  // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowseropenwindow
  DOMString url;
  DOMString target;
  DOMString features;
  // Element frameElement;
};

dictionary BrowserElementVisibilityChangeEventDetail {
  boolean visible;
};

BrowserElement implements BrowserElementCommon;
BrowserElement implements BrowserElementPrivileged;

[NoInterfaceObject, Exposed=(Window,Worker)]
interface BrowserElementCommon {
  [Throws,
   Pref="dom.mozbrowser.enabled"]
  void setVisible(boolean visible);

  [Throws,
   Pref="dom.mozbrowser.enabled"]
  boolean getVisible();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void setActive(boolean active);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //boolean getActive();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void addNextPaintListener(BrowserElementNextPaintEventCallback listener);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void removeNextPaintListener(BrowserElementNextPaintEventCallback listener);
};

[NoInterfaceObject, Exposed=(Window,Worker)]
interface BrowserElementPrivileged {
  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void sendMouseEvent(DOMString type,
  //                    unsigned long x,
  //                    unsigned long y,
  //                    unsigned long button,
  //                    unsigned long clickCount,
  //                    unsigned long modifiers);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled",
  // Func="TouchEvent::PrefEnabled"]
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

  [Func="::dom::window::Window::global_is_mozbrowser", Throws]
  void goBack();

  [Func="::dom::window::Window::global_is_mozbrowser", Throws]
  void goForward();

  [Func="::dom::window::Window::global_is_mozbrowser", Throws]
  void reload(optional boolean hardReload = false);

  [Func="::dom::window::Window::global_is_mozbrowser", Throws]
  void stop();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest download(DOMString url,
  //                    optional BrowserElementDownloadOptions options);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest purgeHistory();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest getScreenshot([EnforceRange] unsigned long width,
  //                         [EnforceRange] unsigned long height,
  //                         optional DOMString mimeType="");

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void zoom(float zoom);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest getCanGoBack();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest getCanGoForward();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest getContentDimensions();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest setInputMethodActive(boolean isActive);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void setNFCFocus(boolean isFocus);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void findAll(DOMString searchString, BrowserFindCaseSensitivity caseSensitivity);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void findNext(BrowserFindDirection direction);

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //void clearMatch();

  //[Throws,
  // Pref="dom.mozBrowserFramesEnabled"]
  //DOMRequest executeScript(DOMString script,
  //                         optional BrowserElementExecuteScriptOptions options);

};
