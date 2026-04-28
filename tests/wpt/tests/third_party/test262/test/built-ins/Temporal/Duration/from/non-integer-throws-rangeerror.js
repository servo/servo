// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: A non-integer value for any recognized property in the property bag, throws a RangeError
features: [Temporal]
---*/

const fields = [
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
];
fields.forEach((field) => {
  assert.throws(RangeError, () => Temporal.Duration.from({ [field]: 1.5 }));
  assert.throws(RangeError, () => Temporal.Duration.from({ [field]: -1.5 }));
});
