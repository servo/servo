// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: CalendarResolveFields throws TypeError before RangeError (chinese calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

// Missing year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.ZonedDateTime.from({ calendar: "chinese", monthCode: "M05", month: 6, day: 1, timeZone: "UTC" }),
  "Missing year throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing month/monthCode should throw TypeError even with invalid day
assert.throws(
  TypeError,
  () => Temporal.ZonedDateTime.from({ calendar: "chinese", year: 2020, day: 32, timeZone: "UTC" }, { overflow: "reject" }),
  "Missing month throws TypeError before out-of-range day throws RangeError"
);

// Missing day should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.ZonedDateTime.from({ calendar: "chinese", year: 2020, monthCode: "M05", month: 6, timeZone: "UTC" }),
  "Missing day throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.ZonedDateTime.from({ calendar: "chinese", year: undefined, monthCode: "M05", month: 6, day: 1, timeZone: "UTC" }),
  "undefined year throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from({ calendar: "chinese", year: 2020, monthCode: "M05", month: 12, day: 1, timeZone: "UTC" }),
  "month/monthCode conflict throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from({ calendar: "chinese", year: 2020, month: 1, day: 32, timeZone: "UTC" }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all types are valid"
);
