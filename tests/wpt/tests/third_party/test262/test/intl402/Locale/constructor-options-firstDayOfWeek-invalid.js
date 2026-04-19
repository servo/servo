// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    x. Let fw be ? GetOption(options, "firstDayOfWeek", "string", undefined, undefined).
    x. If fw is not undefined, then
       x. Set fw to !WeekdayToString(fw).
       x. If fw does not match the type sequence (from UTS 35 Unicode Locale Identifier, section 3.2), throw a RangeError exception.
    ...

features: [Intl.Locale,Intl.Locale-info]
---*/

const invalidFirstDayOfWeekOptions = [
  "",
  "m",
  "mo",
  "longerThan8Chars",
];
for (const firstDayOfWeek of invalidFirstDayOfWeekOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale('en', {firstDayOfWeek});
  }, `new Intl.Locale("en", {firstDayOfWeek: "${firstDayOfWeek}"}) throws RangeError`);
}
