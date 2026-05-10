// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: Negative zero, as an extended year, fails
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
const invalidStrings = [
  "-000000-12-07T03:24:30",
  "-000000-12-07T03:24:30+01:00",
  "-000000-12-07T03:24:30+00:00[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(arg, time),
    "Cannot use minus zero as extended year (first argument)"
  );

  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(time, arg),
    "Cannot use minus zero as extended year (second argument)"
  );
});
