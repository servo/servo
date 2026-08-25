// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.subtract
description: Options argument is ignored.
includes: [temporalHelpers.js]
features: [Symbol, Temporal]
---*/

const values = [
  undefined,
  null,
  true,
  "hello",
  Symbol("foo"),
  1,
  1n,
  {},
  () => {},
  { get overflow() { throw new Test262Error("should not get overflow") } },
];

const time = Temporal.PlainTime.from("15:23:30.123456789");
for (const options of values) {
  TemporalHelpers.assertPlainTime(time.subtract({ hours: 1 }, options),
    14, 23, 30, 123, 456, 789);
}

