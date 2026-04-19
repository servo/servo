// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tostring
description: If calendarName is "auto", "iso8601" should be omitted.
features: [Temporal]
---*/

const monthday = new Temporal.PlainMonthDay(5, 2);
const result = monthday.toString({ calendarName: "auto" });
assert.sameValue(result, "05-02", `built-in ISO calendar for calendarName = auto`);
