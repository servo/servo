// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.supportedLocalesOf
description: Checks handling of an empty locales argument to the supportedLocalesOf function.
info: |
    Intl.DurationFormat.supportedLocalesOf ( locales [, options ])
    (...)
    3. Return ? SupportedLocales(availableLocales, requestedLocales, options).
includes: [compareArray.js]
features: [Intl.DurationFormat]
---*/

assert.sameValue(typeof Intl.DurationFormat.supportedLocalesOf, "function",
                 "Should support Intl.DurationFormat.supportedLocalesOf.");

assert.compareArray(Intl.DurationFormat.supportedLocalesOf(), []);
assert.compareArray(Intl.DurationFormat.supportedLocalesOf([]), []);
