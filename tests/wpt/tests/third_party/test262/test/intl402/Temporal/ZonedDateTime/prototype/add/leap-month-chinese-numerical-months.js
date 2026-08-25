// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: >
  Check mapping of numerical months across leap years. This catches bugs in
  implementations where numeric months are not correctly mapped across leap
  years, allowing additions that should throw with overflow reject.
  See https://github.com/tc39/test262/issues/4905
features: [Temporal]
---*/

// 2012 is a leap year, and month 5 is the leap month 闰四月, inserted after month 4 in 2012.
const instance = Temporal.ZonedDateTime.from({ calendar: "chinese", year: 2012, month: 5, day: 1, timeZone: "UTC" });

assert.throws(
  RangeError,
  () => instance.add("P1Y1M", { overflow: "reject" }),
  "Adding a year and a month to a numerical (leap) month."
);

const oneYear = new Temporal.Duration(1);
assert.throws(
  RangeError,
  () => instance.add(oneYear, { overflow: "reject" }),
  "Adding a year to a numerical (leap) month."
);
