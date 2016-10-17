/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-nodelist
 */

interface NodeList {
  [Pure]
  getter Node? item(unsigned long index);
  [Pure]
  readonly attribute unsigned long length;
  iterable<Node?>;
};
