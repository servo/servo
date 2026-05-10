// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Leap second is a valid ISO string for a calendar in a property bag
features: [Temporal]
---*/

const calendar = "2016-12-31T23:59:60";

const arg = { year: 2019, monthCode: "M06", calendar };
const result1 = Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6));
assert.sameValue(
  result1,
  0,
  "leap second is a valid ISO string for calendar (first argument)"
);
const result2 = Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg);
assert.sameValue(
  result2,
  0,
  "leap second is a valid ISO string for calendar (second argument)"
);
