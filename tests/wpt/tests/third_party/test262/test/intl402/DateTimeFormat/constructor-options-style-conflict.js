// Copyright 2021 Kate Miháliková. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Conflicting properties of dateStyle/timeStyle must be rejected with a TypeError for the options argument to the DateTimeFormat constructor.
info: |
  CreateDateTimeFormat ( newTarget, locales, options, required, defaults )

  1. If _dateStyle_ is not *undefined* or _timeStyle_ is not *undefined*, then
    1. If _hasExplicitFormatComponents_ is *true*, then
      1. Throw a *TypeError* exception.
    1. If _required_ is ~date~ and _timeStyle_ is not *undefined*, then
      1. Throw a *TypeError* exception.
    1. If _required_ is ~time~ and _dateStyle_ is not *undefined*, then
      1. Throw a *TypeError* exception.
---*/


// Table 4 - Property column + example value from Values column
const conflictingOptions = [
 [ "weekday", "short" ],
 [ "era", "short" ],
 [ "year", "numeric" ],
 [ "month", "numeric" ],
 [ "day", "numeric" ],
 [ "dayPeriod", "short" ],
 [ "hour", "numeric" ],
 [ "minute", "numeric" ],
 [ "second", "numeric" ],
 [ "fractionalSecondDigits", 3 ],
 [ "timeZoneName", "short" ],
];

for (const [ option, value ] of conflictingOptions) {
  assert.throws(TypeError, function() {
    new Intl.DateTimeFormat("en", { [option]: value, dateStyle: "short" });
  }, `new Intl.DateTimeFormat("en", { ${option}: "${value}",  dateStyle: "short" }) throws TypeError`);

  assert.throws(TypeError, function() {
    new Intl.DateTimeFormat("en", { [option]: value, timeStyle: "short" });
  }, `new Intl.DateTimeFormat("en", { ${option}: "${value}",  timeStyle: "short" }) throws TypeError`);
}
