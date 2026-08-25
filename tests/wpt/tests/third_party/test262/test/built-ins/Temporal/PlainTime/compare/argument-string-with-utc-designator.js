// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: RangeError thrown if a string with UTC designator is used as a PlainTime
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "2019-10-01T09:00:00Z",
  "2019-10-01T09:00:00Z[UTC]",
  "09:00:00Z[UTC]",
  "09:00:00Z",
];
const plainTime = new Temporal.PlainTime();
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(arg, plainTime),
    "String with UTC designator should not be valid as a PlainTime (first argument)"
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(plainTime, arg),
    "String with UTC designator should not be valid as a PlainTime (second argument)"
  );
});
