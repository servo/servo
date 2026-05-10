// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: CalendarResolveFields throws TypeError before RangeError (gregory calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

const plainDateTime = Temporal.PlainDateTime.from({ calendar: "gregory", year: 2020, month: 5, day: 15, hour: 12 });

// Missing required property (eraYear when era is present) should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => plainDateTime.with({ era: "ce", monthCode: "M05", month: 6 }),
  "Missing eraYear throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing required property (eraYear when era is present) should throw TypeError even with invalid day
assert.throws(
  TypeError,
  () => plainDateTime.with({ era: "ce", day: 32 }),
  "Missing eraYear throws TypeError before out-of-range day throws RangeError"
);

// undefined eraYear should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => plainDateTime.with({ era: "ce", eraYear: undefined, monthCode: "M05", month: 6 }),
  "undefined eraYear throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => plainDateTime.with({ monthCode: "M05", month: 6 }),
  "month/monthCode conflict throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => plainDateTime.with({ day: 32 }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all types are valid"
);
