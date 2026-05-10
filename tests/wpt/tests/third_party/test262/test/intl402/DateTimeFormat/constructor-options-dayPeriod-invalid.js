// Copyright 2019 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
  Checks error cases for the options argument to the DateTimeFormat constructor.
info: |
  [[DayPeriod]]    `"dayPeriod"`    `"narrow"`, `"short"`, `"long"`
  CreateDateTimeFormat ( dateTimeFormat, locales, options, required, defaults )

  ...
features: [Intl.DateTimeFormat-dayPeriod]
---*/

const invalidOptions = [
  "",
  "LONG",
  " long",
  "short ",
  "full",
  "numeric",
];
for (const dayPeriod of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.DateTimeFormat("en", { dayPeriod });
  }, `new Intl.DateTimeFormat("en", { dayPeriod: "${dayPeriod}" }) throws RangeError`);
}
