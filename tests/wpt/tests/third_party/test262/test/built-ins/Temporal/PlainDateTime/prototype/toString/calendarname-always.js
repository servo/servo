// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: If calendarName is "always", the calendar ID should be included.
features: [Temporal]
---*/

const date = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 0, 0, 0, 0);
const result = date.toString({ calendarName: "always" });
assert.sameValue(result, "1976-11-18T15:23:00[u-ca=iso8601]", `built-in ISO calendar for calendarName = always`);
