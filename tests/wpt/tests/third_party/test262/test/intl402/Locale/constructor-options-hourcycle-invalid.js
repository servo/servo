// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    20. Let hc be ? GetOption(options, "hourCycle", "string", « "h11", "h12", "h23", "h24" », undefined).
    ...

    GetOption ( options, property, type, values, fallback )
    ...
    2.  d. If values is not undefined, then
            i. If values does not contain an element equal to value, throw a RangeError exception.
    ...
features: [Intl.Locale]
---*/


const invalidHourCycleOptions = [
  "",
  "h",
  "h00",
  "h01",
  "h10",
  "h13",
  "h22",
  "h25",
  "h48",
  "h012",
  "h120",
  "h12\0",
  "H12",
];
for (const hourCycle of invalidHourCycleOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {hourCycle});
  }, `new Intl.Locale("en", {hourCycle: "${hourCycle}"}) throws RangeError`);
}
