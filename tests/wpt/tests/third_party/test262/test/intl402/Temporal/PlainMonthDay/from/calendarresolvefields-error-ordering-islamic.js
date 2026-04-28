// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: CalendarResolveFields throws TypeError before RangeError (islamic calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

// Missing year (required for month) should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "islamic-civil", monthCode: "M04", month: 5, day: 1 }),
  "Missing year throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing monthCode/month should throw TypeError even with invalid day
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "islamic-civil", year: 1445, day: 32 }, { overflow: "reject" }),
  "Missing monthCode/month throws TypeError before out-of-range day throws RangeError"
);

// Missing day should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "islamic-civil", year: 1445, monthCode: "M04", month: 5 }),
  "Missing day throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "islamic-civil", year: undefined, monthCode: "M04", month: 5, day: 1 }),
  "undefined year throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ calendar: "islamic-civil", year: 1445, monthCode: "M04", month: 5, day: 1 }),
  "month/monthCode conflict throws RangeError when all required fields present"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ calendar: "islamic-civil", year: 1445, monthCode: "M01", day: 32 }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all required fields present"
);
