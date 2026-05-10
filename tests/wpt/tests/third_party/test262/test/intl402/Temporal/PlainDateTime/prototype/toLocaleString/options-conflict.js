// Copyright (C) 2021 Kate Miháliková. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-temporal.plaindatetime.prototype.tolocalestring
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
  [ "weekday", "short" ],
  [ "era", "short" ],
  [ "year", "numeric" ],
  [ "month", "numeric" ],
  [ "day", "numeric" ],
  [ "hour", "numeric" ],
  [ "minute", "numeric" ],
  [ "second", "numeric" ],
  [ "dayPeriod", "short" ],
  [ "fractionalSecondDigits", 3 ],
];
const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

assert.sameValue(typeof datetime.toLocaleString("en", { dateStyle: "short" }), "string");
assert.sameValue(typeof datetime.toLocaleString("en", { timeStyle: "short" }), "string");

for (const [ option, value ] of conflictingOptions) {
  assert.throws(TypeError, function() {
    datetime.toLocaleString("en", { [option]: value, dateStyle: "short" });
  }, `datetime.toLocaleString("en", { ${option}: "${value}",  dateStyle: "short" }) throws TypeError`);

  assert.throws(TypeError, function() {
    datetime.toLocaleString("en", { [option]: value, timeStyle: "short" });
  }, `datetime.toLocaleString("en", { ${option}: "${value}",  timeStyle: "short" }) throws TypeError`);

  // dateStyle or timeStyle present but undefined does not conflict
  datetime.toLocaleString("en", { [option]: value, dateStyle: undefined });
  datetime.toLocaleString("en", { [option]: value, timeStyle: undefined });
}
