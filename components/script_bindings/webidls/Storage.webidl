/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/#the-storage-interface
 *
 */

[Exposed=Window]
interface Storage {

  readonly attribute unsigned long length;

  DOMString? key(unsigned long index);

  getter DOMString? getItem(DOMString name);

  [Throws]
  setter undefined setItem(DOMString name, DOMString value);

  deleter undefined removeItem(DOMString name);

  undefined clear();
};
