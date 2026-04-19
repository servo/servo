// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Reject value for overflow option
features: [Temporal]
---*/

const bad = { year: 2019, month: 13 };
assert.throws(RangeError, () => Temporal.PlainYearMonth.from(bad, { overflow: "reject" }));

[-1, 0, 13, 9995].forEach((month) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.from({year: 2021, month, day: 5}, { overflow: "reject" }),
    `Month ${month} is out of range for 2021 with overflow: reject`
  );
});
