/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#interface-htmlcollection

[LegacyUnenumerableNamedProperties]
interface HTMLCollection {
  [Pure]
  readonly attribute unsigned long length;
  [Pure]
  getter Element? item(unsigned long index);
  [Pure]
  getter Element? namedItem(DOMString name);
};
