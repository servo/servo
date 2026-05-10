// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Errors due to missing properties on fields object are thrown in the correct order
features: [Temporal]
---*/

let getMonth = false;
let getMonthCode = false;
const missingYearAndMonth = {
  get month() {
    getMonth = true;
  },
  get monthCode() {
    getMonthCode = true;
  },
};
assert.throws(TypeError, () => Temporal.PlainYearMonth.from(missingYearAndMonth), "year should be checked after fetching but before resolving the month");
assert(getMonth, "year is fetched after month");
assert(getMonthCode, "year is fetched after monthCode");

assert.throws(TypeError, () => Temporal.PlainYearMonth.from({ year: 2000 }), "month should be resolved last");
