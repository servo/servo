// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks error cases for the locales argument to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    3. Let _requestedLocales_ be ? CanonicalizeLocaleList(_locales_).
includes: [testIntl.js]
features: [Intl.Segmenter]
---*/

for (const [locales, expectedError] of getInvalidLocaleArguments()) {
  assert.throws(expectedError, function() { new Intl.Segmenter(locales) })
}
