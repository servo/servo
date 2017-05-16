/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// A servo-only interface for testing worklets
[Pref="dom.worklet.testing.enabled", Exposed=(Window), Constructor]
interface TestWorklet {
   [NewObject] Promise<void> addModule(USVString moduleURL, optional WorkletOptions options);
   DOMString? lookup(DOMString key);
};
