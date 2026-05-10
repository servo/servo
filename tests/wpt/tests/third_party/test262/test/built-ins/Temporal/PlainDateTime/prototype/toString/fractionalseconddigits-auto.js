// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: auto value for fractionalSecondDigits option
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainDateTime(1976, 2, 4, 5, 3, 1), "1976-02-04T05:03:01"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23), "1976-11-18T15:23:00"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30), "1976-11-18T15:23:30"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 400), "1976-11-18T15:23:30.1234"],
];

for (const [datetime, expected] of tests) {
  assert.sameValue(datetime.toString(), expected, "default is to emit seconds and drop trailing zeroes");
  assert.sameValue(datetime.toString({ fractionalSecondDigits: "auto" }), expected, "auto is the default");
}
