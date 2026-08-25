// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: CalendarResolveFields throws TypeError before RangeError (gregory calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

// Missing year (and no era/eraYear) should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainDate.from({ calendar: "gregory", monthCode: "M05", month: 6, day: 1 }),
  "Missing year/era throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing month/monthCode should throw TypeError even with invalid day
assert.throws(
  TypeError,
  () => Temporal.PlainDate.from({ calendar: "gregory", year: 2020, day: 32 }),
  "Missing month throws TypeError before out-of-range day throws RangeError"
);

// Missing day should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainDate.from({ calendar: "gregory", year: 2020, monthCode: "M05", month: 6 }),
  "Missing day throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainDate.from({ calendar: "gregory", year: undefined, monthCode: "M05", month: 6, day: 1 }),
  "undefined year throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainDate.from({ calendar: "gregory", year: 2020, monthCode: "M05", month: 6, day: 1 }),
  "month/monthCode conflict throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDate.from({ calendar: "gregory", year: 2020, month: 1, day: 32 }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all types are valid"
);
