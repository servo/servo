// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: auto value for fractionalSecondDigits option
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainTime(5, 3, 1), "05:03:01"],
  [new Temporal.PlainTime(15, 23), "15:23:00"],
  [new Temporal.PlainTime(15, 23, 30), "15:23:30"],
  [new Temporal.PlainTime(15, 23, 30, 123, 400), "15:23:30.1234"],
];

for (const [time, expected] of tests) {
  assert.sameValue(time.toString(), expected, "default is to emit seconds and drop trailing zeroes");
  assert.sameValue(time.toString({ fractionalSecondDigits: "auto" }), expected, "auto is the default");
}
