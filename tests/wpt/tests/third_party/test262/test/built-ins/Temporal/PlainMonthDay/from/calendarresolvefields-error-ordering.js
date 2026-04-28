// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: CalendarResolveFields throws TypeError before RangeError
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal]
---*/

// Missing required property (monthCode/month) should throw TypeError even with out-of-range day
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ day: 32 }, { overflow: "reject" }),
  "Missing monthCode/month throws TypeError before out-of-range day throws RangeError"
);

// Missing required property (day) should throw TypeError even with invalid monthCode
assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ monthCode: "M99L" }),
  "Missing day throws TypeError before invalid monthCode throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ monthCode: "M99L", day: 1 }),
  "Invalid monthCode throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ month: 11, monthCode: "M12", day: 18 }),
  "Conflicting month/monthCode throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ monthCode: "M01", day: 32 }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ monthCode: "M00", day: 1 }),
  "Invalid monthCode M00 throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ month: 13, day: 1 }, { overflow: "reject" }),
  "Out-of-range month throws RangeError when all types are valid"
);
