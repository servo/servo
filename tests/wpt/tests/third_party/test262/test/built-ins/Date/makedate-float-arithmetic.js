// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: Combination of arguments producing f64-unrepresentable epoch milliseconds
info: |
  These arguments produce a date which is far out of range in any case, but may
  violate internal assertions in MakeDate if not implemented in the correct
  arithmetic space; see https://github.com/tc39/ecma262/issues/3992.
---*/

const date = new Date(2000, 0, 106752348147);

assert.sameValue(date.getTime(), NaN, "time value outside the valid range is clipped to NaN");
assert.sameValue(date.valueOf(), NaN, "time value outside the valid range is clipped to NaN");
