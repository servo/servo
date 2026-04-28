// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks error cases for the locales argument to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
    1. Let _requestedLocales_ be ? CanonicalizeLocaleList(_locales_).
includes: [testIntl.js]
features: [Intl.ListFormat]
---*/

for (const [locales, expectedError] of getInvalidLocaleArguments()) {
    assert.throws(expectedError, function() { new Intl.ListFormat(locales) })
}
