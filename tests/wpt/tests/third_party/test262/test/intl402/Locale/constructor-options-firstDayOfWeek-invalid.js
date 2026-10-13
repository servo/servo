// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    22. Let fw be ? GetOption(options, "firstDayOfWeek", string, empty, undefined).
    23. If fw is not undefined, then
       a. Set fw to WeekdayToUValue(fw).
       b. If fw cannot be matched by the type Unicode locale nonterminal, throw a RangeError exception.
    ...

features: [Intl.Locale,Intl.Locale-info]
---*/

const invalidFirstDayOfWeekOptions = [
  "",
  "m",
  "mo",
  "longerThan8Chars",
  "abc\0abc",
  "abc?",
  "äöü",
  "\u6161bc",
  8,
  9,
  10,
  -1,
  -10,
  -100,
  -1000,
  8n,
  9n,
  10n,
  -1n,
  -10n,
  -100n,
  -1000n,
  0.5,
  Number.MIN_VALUE,
  -Infinity,
];
for (const firstDayOfWeek of invalidFirstDayOfWeekOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {firstDayOfWeek});
  }, `new Intl.Locale("en", {firstDayOfWeek: "${firstDayOfWeek}"}) throws RangeError`);
}
