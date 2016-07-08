/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#navigator
[Exposed=(Window,Worker)]
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator implements NavigatorID;
Navigator implements NavigatorBluetooth;
Navigator implements NavigatorLanguage;
//Navigator implements NavigatorOnLine;
//Navigator implements NavigatorContentUtils;
//Navigator implements NavigatorStorageUtils;
Navigator implements NavigatorPlugins;

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

[NoInterfaceObject, Exposed=(Window,Worker)]
interface NavigatorBluetooth {
    readonly attribute Bluetooth bluetooth;
};

// https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#navigator-service-worker
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
[NoInterfaceObject, Exposed=(Window,Worker)]
interface NavigatorPlugins {
  [SameObject] readonly attribute PluginArray plugins;
  [SameObject] readonly attribute MimeTypeArray mimeTypes;
  boolean javaEnabled();
};
