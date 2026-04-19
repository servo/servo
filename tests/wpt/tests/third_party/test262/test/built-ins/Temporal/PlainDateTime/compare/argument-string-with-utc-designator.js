// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: RangeError thrown if a string with UTC designator is used as a PlainDateTime
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "2019-10-01T09:00:00Z",
  "2019-10-01T09:00:00Z[UTC]",
];
const dateTime = new Temporal.PlainDateTime(2000, 5, 2);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.compare(arg, dateTime),
    "String with UTC designator should not be valid as a PlainDateTime (first argument)"
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.compare(arg, dateTime),
    "String with UTC designator should not be valid as a PlainDateTime (second argument)"
  );
});
