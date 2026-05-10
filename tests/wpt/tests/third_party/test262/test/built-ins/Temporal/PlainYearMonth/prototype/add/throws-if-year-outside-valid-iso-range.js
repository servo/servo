// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: >
  Throws if thisFields is not within valid ISO date range.
info: |
  Temporal.PlainYearMonth.prototype.add ( temporalDurationLike [ , options ] )

  ...
  3. Return ? AddDurationToYearMonth(add, yearMonth, temporalDurationLike, options).

  AddDurationToYearMonth ( operation, yearMonth, temporalDurationLike, options )

  ...
  8. Set fields.[[Day]] to 1.
  9. Let intermediateDate be ? CalendarDateFromFields(calendar, fields, constrain).
  ...

features: [Temporal]
---*/

const minYearMonth = new Temporal.PlainYearMonth(-271821, 4);
const blank = new Temporal.Duration();

assert.throws(RangeError, () => minYearMonth.add(blank));
