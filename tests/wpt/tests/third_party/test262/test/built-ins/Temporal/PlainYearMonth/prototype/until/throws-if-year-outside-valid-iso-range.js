// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: >
  Throws if thisFields is not within valid ISO date range.
info: |
  Temporal.PlainYearMonth.prototype.until ( other [ , options ] )

  ...
  3. Return ? DifferenceTemporalPlainYearMonth(until, yearMonth, other, options).

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
  minYearMonth.until(minYearMonth),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "minYearMonth.until(minYearMonth)"
);

assert.throws(
  RangeError,
  () => minYearMonth.until(maxYearMonth),
  "minYearMonth.until(maxYearMonth)"
);

assert.throws(
  RangeError,
  () => minYearMonth.until(epochYearMonth),
  "minYearMonth.until(epochYearMonth)"
);
