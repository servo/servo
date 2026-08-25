// Copyright 2018 the V8 project authors, Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.supportedLocalesOf
description: Checks handling of an empty locales argument to the supportedLocalesOf function.
info: |
    Intl.Segmenter.supportedLocalesOf ( locales [, options ])

    3. Return ? SupportedLocales(availableLocales, requestedLocales, options).
includes: [compareArray.js]
features: [Intl.Segmenter]
---*/

assert.sameValue(typeof Intl.Segmenter.supportedLocalesOf, "function",
                 "Should support Intl.Segmenter.supportedLocalesOf.");

assert.compareArray(Intl.Segmenter.supportedLocalesOf(), []);
assert.compareArray(Intl.Segmenter.supportedLocalesOf([]), []);
