// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: CalendarResolveFields throws TypeError before RangeError (japanese calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

const plainYearMonth = Temporal.PlainYearMonth.from({ calendar: "japanese", year: 2020, month: 5 });

// Missing required property (eraYear when era is present) should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => plainYearMonth.with({ era: "heisei", monthCode: "M05", month: 6 }),
  "Missing eraYear throws TypeError before month/monthCode conflict throws RangeError"
);

// undefined eraYear should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => plainYearMonth.with({ era: "heisei", eraYear: undefined, monthCode: "M05", month: 6 }),
  "undefined eraYear throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => plainYearMonth.with({ monthCode: "M05", month: 6 }),
  "month/monthCode conflict throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => plainYearMonth.with({ month: 13 }, { overflow: "reject" }),
  "Out-of-range month throws RangeError when all types are valid"
);
