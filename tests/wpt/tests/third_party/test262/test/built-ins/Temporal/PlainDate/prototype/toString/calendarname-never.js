// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tostring
description: If calendarName is "never", the calendar ID should be omitted.
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
const result = date.toString({ calendarName: "never" });
assert.sameValue(result, "2000-05-02", `built-in ISO calendar for calendarName = never`);
