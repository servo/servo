/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#datatransferitemlist

[Exposed=Window]
interface DataTransferItemList {
  readonly attribute unsigned long length;
  getter DataTransferItem (unsigned long index);
  [Throws] DataTransferItem? add(DOMString data, DOMString type);
  [Throws] DataTransferItem? add(File data);
  [Throws] undefined remove(unsigned long index);
  undefined clear();
};
