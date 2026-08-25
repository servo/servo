// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Non-ASCII minus sign is not acceptable
features: [Temporal]
---*/

const invalidStrings = [
  "1976-11-18T15:23:30.12\u221202:00",
  "\u2212009999-11-18T15:23:30.12",
];

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6)),
    `variant minus sign: ${arg} (first argument)`
  );

  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg),
    `variant minus sign: ${arg} (second argument)`
  );
});
