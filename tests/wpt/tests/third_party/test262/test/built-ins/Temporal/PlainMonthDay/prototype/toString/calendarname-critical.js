// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tostring
description: >
  If calendarName is "calendar", the calendar ID should be included and prefixed
  with "!".
features: [Temporal]
---*/

const monthday = new Temporal.PlainMonthDay(5, 2);
const result = monthday.toString({ calendarName: "critical" });
assert.sameValue(result, "1972-05-02[!u-ca=iso8601]", `built-in ISO calendar for calendarName = critical`);
