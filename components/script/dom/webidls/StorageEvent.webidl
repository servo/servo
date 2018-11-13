/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * Interface for a client side storage. See
 * https://html.spec.whatwg.org/multipage/#the-storageevent-interface
 * for more information.
 *
 * Event sent to a window when a storage area changes.
 */

[Constructor(DOMString type, optional StorageEventInit eventInitDict), Exposed=Window]
interface StorageEvent : Event {
  readonly attribute DOMString? key;
  readonly attribute DOMString? oldValue;
  readonly attribute DOMString? newValue;
  readonly attribute DOMString url;
  readonly attribute Storage? storageArea;
};

dictionary StorageEventInit : EventInit {
  DOMString? key = null;
  DOMString? oldValue = null;
  DOMString? newValue = null;
  DOMString url = "";
  Storage? storageArea = null;
};
