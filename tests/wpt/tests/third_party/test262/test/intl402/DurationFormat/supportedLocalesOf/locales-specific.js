// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.supportedLocalesOf
description: Checks handling of specific locales arguments to the supportedLocalesOf function.
info: |
    Intl.DurationFormat.supportedLocalesOf ( locales [, options ])
    (...)
    3. Return ? SupportedLocales(availableLocales, requestedLocales, options).
includes: [compareArray.js]
locale: [sr, sr-Thai-RS, de, zh-CN]
features: [Intl.DurationFormat]
---*/

assert.sameValue(typeof Intl.DurationFormat.supportedLocalesOf, "function",
                 "Should support Intl.DurationFormat.supportedLocalesOf.");

assert.compareArray(Intl.DurationFormat.supportedLocalesOf("sr"), ["sr"]);

const multiLocale = ["sr-Thai-RS", "de", "zh-CN"];
assert.compareArray(Intl.DurationFormat.supportedLocalesOf(multiLocale, {localeMatcher: "lookup"}), multiLocale);
