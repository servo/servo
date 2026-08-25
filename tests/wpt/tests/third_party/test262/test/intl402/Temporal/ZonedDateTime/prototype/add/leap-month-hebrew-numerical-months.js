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

// 5000 is a leap year, and month 6 is the leap month Adar I, inserted between
// month 5 and 6 of a common year.
const instance = Temporal.ZonedDateTime.from({ calendar: "hebrew", timeZone: "UTC", year: 5000, month: 6, day: 1, hour: 12 });

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



