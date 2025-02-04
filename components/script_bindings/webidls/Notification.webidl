/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://notifications.spec.whatwg.org/#api
[Exposed=(Window,Worker), Pref="dom_notification_enabled"]
interface Notification : EventTarget {
  [Throws]
  constructor(DOMString title, optional NotificationOptions options = {});

  [GetterThrows]
  static readonly attribute NotificationPermission permission;

  [Exposed=Window]
  static Promise<NotificationPermission> requestPermission(optional NotificationPermissionCallback deprecatedCallback);

  static readonly attribute unsigned long maxActions;

  attribute EventHandler onclick;
  attribute EventHandler onshow;
  attribute EventHandler onerror;
  attribute EventHandler onclose;

  [Pure]
  readonly attribute DOMString title;

  [Pure]
  readonly attribute NotificationDirection dir;

  [Pure]
  readonly attribute DOMString lang;

  [Pure]
  readonly attribute DOMString body;

  [Constant]
  readonly attribute DOMString tag;

  [Pure]
  readonly attribute USVString image;

  [Pure]
  readonly attribute USVString icon;

  [Pure]
  readonly attribute USVString badge;

  readonly attribute boolean renotify;

  [Constant]
  readonly attribute boolean requireInteraction;

  [Constant]
  readonly attribute boolean? silent;

  // [Cached, Frozen, Pure]
  readonly attribute /*FrozenArray<<unsigned long>*/any vibrate;

  readonly attribute EpochTimeStamp timestamp;

  [Constant]
  readonly attribute any data;

  // [Cached, Frozen, Pure]
  readonly attribute /*FrozenArray<NotificationAction>*/any actions;

  undefined close();
};

// <https://w3c.github.io/hr-time/#dom-epochtimestamp>
typedef unsigned long long EpochTimeStamp;
typedef (unsigned long or sequence<unsigned long>) VibratePattern;

dictionary NotificationOptions {
  NotificationDirection dir = "auto";
  DOMString lang = "";
  DOMString body = "";
  DOMString tag = "";
  USVString image;
  USVString icon;
  USVString badge;
  VibratePattern vibrate;
  EpochTimeStamp timestamp;
  boolean renotify = false;
  boolean? silent = null;
  boolean requireInteraction = false;
  any data = null;
  sequence<NotificationAction> actions = [];
};

dictionary GetNotificationOptions {
  DOMString tag = "";
};

enum NotificationPermission {
  "default",
  "denied",
  "granted"
};

callback NotificationPermissionCallback = undefined (NotificationPermission permission);

enum NotificationDirection {
  "auto",
  "ltr",
  "rtl"
};

dictionary NotificationAction {
  required DOMString action;
  required DOMString title;
  USVString icon;
};
