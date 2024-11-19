/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#navigator
[Exposed=Window]
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator includes NavigatorID;
Navigator includes NavigatorLanguage;
//Navigator includes NavigatorOnLine;
//Navigator includes NavigatorContentUtils;
//Navigator includes NavigatorStorageUtils;
Navigator includes NavigatorPlugins;
Navigator includes NavigatorCookies;
Navigator includes NavigatorGPU;
Navigator includes NavigatorConcurrentHardware;

// https://html.spec.whatwg.org/multipage/#navigatorid
[Exposed=(Window,Worker)]
interface mixin NavigatorID {
  readonly attribute DOMString appCodeName; // constant "Mozilla"
  readonly attribute DOMString appName;
  readonly attribute DOMString appVersion;
  readonly attribute DOMString platform;
  readonly attribute DOMString product; // constant "Gecko"
  [Exposed=Window] readonly attribute DOMString productSub;
  boolean taintEnabled(); // constant false
  readonly attribute DOMString userAgent;
  [Exposed=Window] readonly attribute DOMString vendor;
  [Exposed=Window] readonly attribute DOMString vendorSub; // constant ""
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
[Exposed=(Window,Worker)]
interface mixin NavigatorLanguage {
  readonly attribute DOMString language;
  readonly attribute any languages;
};

// https://html.spec.whatwg.org/multipage/#navigatorplugins
interface mixin NavigatorPlugins {
  [SameObject] readonly attribute PluginArray plugins;
  [SameObject] readonly attribute MimeTypeArray mimeTypes;
  boolean javaEnabled();
};

// https://html.spec.whatwg.org/multipage/#navigatorcookies
interface mixin NavigatorCookies {
  readonly attribute boolean cookieEnabled;
};

// https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
[Exposed=(Window)]
partial interface Navigator {
  [Pref="dom.permissions.enabled"] readonly attribute Permissions permissions;
};

// https://w3c.github.io/gamepad/#navigator-interface-extension
partial interface Navigator {
  [Pref="dom.gamepad.enabled"] sequence<Gamepad?> getGamepads();
};

// https://html.spec.whatwg.org/multipage/#navigatorconcurrenthardware
interface mixin NavigatorConcurrentHardware {
  readonly attribute unsigned long long hardwareConcurrency;
};
