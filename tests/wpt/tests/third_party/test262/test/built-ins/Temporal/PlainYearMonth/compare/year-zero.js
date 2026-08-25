// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Negative zero, as an extended year, fails
esid: sec-temporal.plainyearmonth.compare
features: [Temporal]
---*/

const ok = new Temporal.PlainYearMonth(2000, 5);
const invalidStrings = [
  "-000000-06",
  "-000000-06-24",
  "-000000-06-24T15:43:27",
  "-000000-06-24T15:43:27+01:00",
  "-000000-06-24T15:43:27+00:00[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.compare(arg, ok),
    "Cannot use minus zero as extended year (first argument)"
  );

  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.compare(ok, arg),
    "Cannot use minus zero as extended year (second argument)"
  );
});
