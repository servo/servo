// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tostring
description: >
  If calendarName is "calendar", the calendar ID should be included and prefixed
  with "!".
features: [Temporal]
---*/

const yearmonth = new Temporal.PlainYearMonth(2000, 5);
const result = yearmonth.toString({ calendarName: "critical" });
assert.sameValue(result, "2000-05-01[!u-ca=iso8601]", `built-in ISO calendar for calendarName = critical`);
