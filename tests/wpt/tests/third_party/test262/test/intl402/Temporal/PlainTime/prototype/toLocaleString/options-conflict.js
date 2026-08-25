// Copyright (C) 2021 Kate Miháliková. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-temporal.plaintime.prototype.tolocalestring
description: >
    Conflicting properties of dateStyle must be rejected with a TypeError for the options argument
info: |
    Using sec-temporal-getdatetimeformatpattern:
    GetDateTimeFormatPattern ( dateStyle, timeStyle, matcher, opt, dataLocaleData, hc )

    1. If dateStyle is not undefined or timeStyle is not undefined, then
      a. For each row in Table 7, except the header row, do
        i. Let prop be the name given in the Property column of the row.
        ii. Let p be opt.[[<prop>]].
        iii. If p is not undefined, then
          1. Throw a TypeError exception.
features: [Temporal]
---*/

// Table 14 - Supported fields + example value for each field
const conflictingOptions = [
  [ "hour", "numeric" ],
  [ "minute", "numeric" ],
  [ "second", "numeric" ],
  [ "dayPeriod", "short" ],
  [ "fractionalSecondDigits", 3 ],
];
const time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

assert.sameValue(typeof time.toLocaleString("en", { timeStyle: "short" }), "string");

assert.throws(TypeError, function () {
  time.toLocaleString("en", { dateStyle: "short" });
}, "dateStyle conflicts with PlainTime");

for (const [ option, value ] of conflictingOptions) {
  assert.throws(TypeError, function() {
    time.toLocaleString("en", { [option]: value, timeStyle: "short" });
  }, `time.toLocaleString("en", { ${option}: "${value}",  timeStyle: "short" }) throws TypeError`);

  // dateStyle or timeStyle present but undefined does not conflict
  time.toLocaleString("en", { [option]: value, dateStyle: undefined });
  time.toLocaleString("en", { [option]: value, timeStyle: undefined });
}
