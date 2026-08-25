// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: CalendarResolveFields throws TypeError before RangeError (gregory calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

// Missing year (and no era/eraYear) should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "gregory", monthCode: "M05", month: 6 }),
  "Missing year/era throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "gregory", year: undefined, monthCode: "M05", month: 6 }),
  "undefined year throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ calendar: "gregory", year: 2020, monthCode: "M05", month: 6 }),
  "month/monthCode conflict throws RangeError when all types are valid"
);
