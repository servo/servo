// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: CalendarResolveFields throws TypeError before RangeError (chinese calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

// Missing year (required for month) should throw TypeError even with month/monthCode conflict
// This is the example from https://github.com/tc39/proposal-intl-era-monthcode/issues/90#issuecomment-3518215482
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "chinese", monthCode: "M04", month: 5, day: 1 }),
  "Missing year throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing monthCode/month should throw TypeError even with invalid day
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "chinese", year: 2020, day: 32 }, { overflow: "reject" }),
  "Missing monthCode/month throws TypeError before out-of-range day throws RangeError"
);

// Missing day should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "chinese", year: 2020, monthCode: "M04", month: 5 }),
  "Missing day throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined year should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ calendar: "chinese", year: undefined, monthCode: "M04", month: 5, day: 1 }),
  "undefined year throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ calendar: "chinese", year: 2020, monthCode: "M04", month: 5, day: 1 }),
  "month/monthCode conflict throws RangeError when all required fields present"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ calendar: "chinese", year: 2020, monthCode: "M01", day: 32 }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all required fields present"
);

// No RangeError if month is present and reference date has a different ordinal month.
var pmd = Temporal.PlainMonthDay.from({ calendar: "chinese", year: 2004, monthCode: "M04", month: 5, day: 1 });
var pd = Temporal.PlainDate.from(pmd.toString());
assert.sameValue(pmd.monthCode, "M04", "Temporal.PlainMonthDay monthCode");
assert.sameValue(pd.monthCode, "M04", "Temporal.PlainDate monthCode");
assert.sameValue(pd.month, 4, "Temporal.PlainDate ordinal month");
