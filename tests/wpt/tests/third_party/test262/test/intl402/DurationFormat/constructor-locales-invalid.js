// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks error cases for the locales argument to the DurationFormat constructor.
info: |
    Intl.DurationFormat  ( [ locales [ , options ] ] )
    (...)
    3. Let _requestedLocales_ be ? CanonicalizeLocaleList(_locales_).
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

for (const [locales, expectedError] of getInvalidLocaleArguments()) {
    assert.throws(expectedError, function() { new Intl.DurationFormat(locales) })
}
