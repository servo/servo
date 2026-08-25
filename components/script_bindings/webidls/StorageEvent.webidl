/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * Interface for a client side storage. See
 * https://html.spec.whatwg.org/multipage/#the-storageevent-interface
 * for more information.
 *
 * Event sent to a window when a storage area changes.
 */

[Exposed=Window]
interface StorageEvent : Event {
  [Throws] constructor(DOMString type, optional StorageEventInit eventInitDict = {});
  readonly attribute DOMString? key;
  readonly attribute DOMString? oldValue;
  readonly attribute DOMString? newValue;
  readonly attribute DOMString url;
  readonly attribute Storage? storageArea;


  undefined initStorageEvent(DOMString type, optional boolean bubbles = false,
  optional boolean cancelable = false, optional DOMString? key = null, optional
  DOMString? oldValue = null, optional DOMString? newValue = null, optional
  USVString url = "", optional Storage? storageArea = null);
};

dictionary StorageEventInit : EventInit {
  DOMString? key = null;
  DOMString? oldValue = null;
  DOMString? newValue = null;
  DOMString url = "";
  Storage? storageArea = null;
};
