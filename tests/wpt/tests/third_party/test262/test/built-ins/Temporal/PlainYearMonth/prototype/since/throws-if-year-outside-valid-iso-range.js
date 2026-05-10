// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Throws if thisFields is not within valid ISO date range.
info: |
  Temporal.PlainYearMonth.prototype.since ( other [ , options ] )

  ...
  3. Return ? DifferenceTemporalPlainYearMonth(since, yearMonth, other, options).

  DifferenceTemporalPlainYearMonth ( operation, yearMonth, other, options )

  ...
  8. Set thisFields.[[Day]] to 1.
  9. Let thisDate be ? CalendarDateFromFields(calendar, thisFields, constrain).
  ...

includes: [temporalHelpers.js]
features: [Temporal]
---*/

const minYearMonth = new Temporal.PlainYearMonth(-271821, 4);
const maxYearMonth = new Temporal.PlainYearMonth(275760, 9);
const epochYearMonth = new Temporal.PlainYearMonth(1970, 1);

TemporalHelpers.assertDuration(
  minYearMonth.since(minYearMonth),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "minYearMonth.since(minYearMonth)"
);

assert.throws(
  RangeError,
  () => minYearMonth.since(maxYearMonth),
  "minYearMonth.since(maxYearMonth)"
);

assert.throws(
  RangeError,
  () => minYearMonth.since(epochYearMonth),
  "minYearMonth.since(epochYearMonth)"
);
