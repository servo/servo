// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: CalendarResolveFields throws TypeError before RangeError
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal]
---*/

// Missing required property (year) should throw TypeError even with invalid monthCode
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ monthCode: "M99L" }),
  "Missing year throws TypeError before invalid monthCode throws RangeError"
);

// Missing required property (month/monthCode) should throw TypeError even with valid year
assert.throws(
  TypeError,
  () => Temporal.PlainYearMonth.from({ year: 2021 }),
  "Missing month/monthCode throws TypeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M99L" }),
  "Invalid monthCode throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, month: 11, monthCode: "M12" }),
  "Conflicting month/monthCode throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, month: 13 }, { overflow: "reject" }),
  "Out-of-range month throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M00" }),
  "Invalid monthCode M00 throws RangeError when all types are valid"
);
