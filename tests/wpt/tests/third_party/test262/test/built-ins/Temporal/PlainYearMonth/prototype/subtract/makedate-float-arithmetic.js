// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Combination of arguments producing f64-unrepresentable epoch milliseconds
info: |
  These arguments produce a date which is far out of range in any case, but may
  violate internal assertions in MakeDate if not implemented in the correct
  arithmetic space; see https://github.com/tc39/ecma262/issues/3992.
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 1);

const cases = [
  [{ years: 292278002 }, "years"],
  [{ years: -292278002 }, "negative years"],
];

for (const [arg, descr] of cases) {
  assert.throws(RangeError, () => instance.subtract(arg), `${descr} put the result out of range`);
}
