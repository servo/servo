// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Throw a RangeError if fields conflict with each other
features: [Temporal, Intl.Era-monthcode]
---*/

// eraYear and year must be consistent when monthCode is present.
{
  let fields = {
    calendar: "gregory",
    era: "ce",
    eraYear: 2024,
    year: 2023,
    monthCode: "M01",
    day: 1,
  };
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from(fields));
}

// eraYear and year must be consistent when month is present.
{
  let fields = {
    calendar: "gregory",
    era: "ce",
    eraYear: 2024,
    year: 2023,
    month: 1,
    day: 1,
  };
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from(fields));
}

// monthCode and month must be consistent.
{
  let fields = {
    calendar: "gregory",
    year: 2024,
    monthCode: "M01",
    month: 2,
    day: 1,
  };
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from(fields));
}

