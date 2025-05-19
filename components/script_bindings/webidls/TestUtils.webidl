/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_TESTBINDING

// https://testutils.spec.whatwg.org/

[Exposed=(Window,Worker), Pref="dom_testutils_enabled"]
namespace TestUtils {
  [NewObject] Promise<undefined> gc();
};
