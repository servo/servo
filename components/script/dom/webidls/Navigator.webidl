/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#navigator
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator implements NavigatorID;
Navigator implements NavigatorLanguage;
//Navigator implements NavigatorOnLine;
//Navigator implements NavigatorContentUtils;
//Navigator implements NavigatorStorageUtils;
Navigator implements NavigatorPlugins;
Navigator implements NavigatorCookies;

// https://html.spec.whatwg.org/multipage/#navigatorid
[NoInterfaceObject, Exposed=(Window,Worker)]
interface NavigatorID {
  readonly attribute DOMString appCodeName; // constant "Mozilla"
  readonly attribute DOMString appName;
  readonly attribute DOMString appVersion;
  readonly attribute DOMString platform;
  readonly attribute DOMString product; // constant "Gecko"
  boolean taintEnabled(); // constant false
  readonly attribute DOMString userAgent;
};

// https://webbluetoothcg.github.io/web-bluetooth/#navigator-extensions
partial interface Navigator {
  [SameObject, Pref="dom.bluetooth.enabled"] readonly attribute Bluetooth bluetooth;
};

// https://w3c.github.io/ServiceWorker/#navigator-service-worker
partial interface Navigator {
  [SameObject, Pref="dom.serviceworker.enabled"] readonly attribute ServiceWorkerContainer serviceWorker;
};

// https://html.spec.whatwg.org/multipage/#navigatorlanguage
[NoInterfaceObject, Exposed=(Window,Worker)]
interface NavigatorLanguage {
  readonly attribute DOMString language;
  // https://github.com/servo/servo/issues/10073
  //readonly attribute DOMString[] languages;
};

// https://html.spec.whatwg.org/multipage/#navigatorplugins
[NoInterfaceObject]
interface NavigatorPlugins {
  [SameObject] readonly attribute PluginArray plugins;
  [SameObject] readonly attribute MimeTypeArray mimeTypes;
  boolean javaEnabled();
};

// https://html.spec.whatwg.org/multipage/#navigatorcookies
[NoInterfaceObject]
interface NavigatorCookies {
  readonly attribute boolean cookieEnabled;
};

// https://w3c.github.io/webvr/spec/1.1/#interface-navigator
partial interface Navigator {
  [Pref="dom.webvr.enabled"] Promise<sequence<VRDisplay>> getVRDisplays();
};

// https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
[Exposed=(Window)]
partial interface Navigator {
  [Pref="dom.permissions.enabled"] readonly attribute Permissions permissions;
};

// https://w3c.github.io/gamepad/#navigator-interface-extension
partial interface Navigator {
    [Pref="dom.gamepad.enabled"] GamepadList getGamepads();
};
