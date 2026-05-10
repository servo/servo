// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: CalendarResolveFields throws TypeError before RangeError (japanese calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

// Missing year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", monthCode: "M05", month: 12 }),
  "Missing year throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing month/monthCode should throw TypeError even with valid year
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", year: 2020 }),
  "Missing month/monthCode throws TypeError"
);

// undefined year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", year: undefined, monthCode: "M05", month: 12 }),
  "undefined year throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined month should throw TypeError even with valid year
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", year: 2020, month: undefined }),
  "undefined month throws TypeError"
);

// undefined monthCode (when month is also missing) should throw TypeError
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", year: 2020, monthCode: undefined }),
  "undefined monthCode throws TypeError when month is missing"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", year: 2020, monthCode: "M05", month: 12 }),
  "month/monthCode conflict throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ calendar: "japanese", year: 2020, month: 13 }, { overflow: "reject" }),
  "Out-of-range month throws RangeError when all types are valid"
);
