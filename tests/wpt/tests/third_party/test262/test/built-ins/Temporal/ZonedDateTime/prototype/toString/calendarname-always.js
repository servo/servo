// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: If calendarName is "always", the calendar ID should be included.
features: [Temporal]
---*/

const date = new Temporal.ZonedDateTime(3661_987_654_321n, "UTC");
const result = date.toString({ calendarName: "always" });
assert.sameValue(result, "1970-01-01T01:01:01.987654321+00:00[UTC][u-ca=iso8601]", `built-in ISO calendar for calendarName = always`);
