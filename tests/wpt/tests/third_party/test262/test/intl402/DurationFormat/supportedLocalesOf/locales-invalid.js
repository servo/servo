// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.supportedLocalesOf
description: Checks error cases for the locales argument to the supportedLocalesOf function.
info: |
    Intl.DurationFormat.supportedLocalesOf ( locales [, options ])
    (...)
    2. Let requestedLocales be CanonicalizeLocaleList(locales).
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

assert.sameValue(typeof Intl.DurationFormat.supportedLocalesOf, "function",
                 "Should support Intl.DurationFormat.supportedLocalesOf.");

for (const [locales, expectedError] of getInvalidLocaleArguments()) {
    assert.throws(expectedError, () => Intl.DurationFormat.supportedLocalesOf(locales));
}
