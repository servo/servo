// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: A plain object argument needs to specify a month
features: [Temporal]
---*/

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from({year: 1976, month: 11, monthCode: "M12", day: 18}),
  "month and monthCode must agree"
);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.from({year: 1976, month: undefined, monthCode: undefined, day: 18}),
  "required prop undefined throws"
);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.from({year: 1976, day: 18, hour: 15, minute: 23, second: 30, millisecond: 123}),
  "required prop missing throws"
);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.from({year: 1976, months: 11, day: 18}),
  "plural \"months\" is not acceptable"
);
