// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: RangeError thrown if a string with UTC designator is used as a PlainYearMonth
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "2019-10-01T09:00:00Z",
  "2019-10-01T09:00:00Z[UTC]",
];
const instance = new Temporal.PlainYearMonth(2000, 5);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => instance.since(arg),
    "String with UTC designator should not be valid as a PlainYearMonth"
  );
});
