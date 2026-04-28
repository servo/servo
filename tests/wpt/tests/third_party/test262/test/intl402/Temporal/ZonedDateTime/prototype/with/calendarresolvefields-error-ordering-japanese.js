// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: CalendarResolveFields throws TypeError before RangeError (japanese calendar)
info: |
  CalendarResolveFields validates field types before validating field ranges,
  ensuring TypeError is thrown before RangeError when both conditions exist.
features: [Temporal, Intl.Era-monthcode]
---*/

const zonedDateTime = Temporal.ZonedDateTime.from({ calendar: "japanese", timeZone: "Asia/Tokyo", year: 2020, month: 5, day: 15, hour: 12 });

// Missing required property (eraYear when era is present) should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => zonedDateTime.with({ era: "heisei", monthCode: "M05", month: 6 }),
  "Missing eraYear throws TypeError before month/monthCode conflict throws RangeError"
);

// Missing required property (eraYear when era is present) should throw TypeError even with invalid day
assert.throws(
  TypeError,
  () => zonedDateTime.with({ era: "heisei", day: 32 }),
  "Missing eraYear throws TypeError before out-of-range day throws RangeError"
);

// undefined eraYear should throw TypeError even with month/monthCode conflict
assert.throws(
  TypeError,
  () => zonedDateTime.with({ era: "heisei", eraYear: undefined, monthCode: "M05", month: 6 }),
  "undefined eraYear throws TypeError before month/monthCode conflict throws RangeError"
);

// After type validation passes, range validation should throw RangeError
assert.throws(
  RangeError,
  () => zonedDateTime.with({ monthCode: "M05", month: 6 }),
  "month/monthCode conflict throws RangeError when all types are valid"
);

assert.throws(
  RangeError,
  () => zonedDateTime.with({ day: 32 }, { overflow: "reject" }),
  "Out-of-range day throws RangeError when all types are valid"
);
