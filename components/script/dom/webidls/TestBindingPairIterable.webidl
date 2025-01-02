/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.

[Pref="dom.testbinding.enabled", Exposed=(Window,Worker)]
interface TestBindingPairIterable {
  [Throws] constructor();
  undefined add(DOMString key, unsigned long value);
  iterable<DOMString, unsigned long>;
};
