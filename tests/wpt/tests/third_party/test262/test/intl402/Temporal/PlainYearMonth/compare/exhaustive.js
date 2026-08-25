// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Tests for compare() with each possible outcome
features: [Temporal]
---*/

const cal1 = "iso8601";
const cal2 = "gregory";

assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2000, 5, cal1), new Temporal.PlainYearMonth(1987, 5, cal2)),
  1,
  "year >"
);
assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(1981, 12, cal1), new Temporal.PlainYearMonth(2048, 12, cal2)),
  -1,
  "year <"
);
assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2000, 5, cal1), new Temporal.PlainYearMonth(2000, 3, cal2)),
  1,
  "month >"
);
assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(1981, 4, cal1), new Temporal.PlainYearMonth(1981, 12, cal2)),
  -1,
  "month <"
);
assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2000, 5, cal1, 30), new Temporal.PlainYearMonth(2000, 5, cal2, 14)),
  1,
  "reference day >"
);
assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(1981, 4, cal1, 1), new Temporal.PlainYearMonth(1981, 4, cal2, 12)),
  -1,
  "reference day <"
);
assert.sameValue(
  Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2000, 5, cal1), new Temporal.PlainYearMonth(2000, 5, cal2)),
  0,
  "="
);
