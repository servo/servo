// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks error cases for the locales argument to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    3. Let _requestedLocales_ be ? CanonicalizeLocaleList(_locales_).
includes: [testIntl.js]
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(typeof Intl.RelativeTimeFormat, "function");

for (const [locales, expectedError] of getInvalidLocaleArguments()) {
    assert.throws(expectedError, function() { new Intl.RelativeTimeFormat(locales) });
}
