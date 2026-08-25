// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: If calendarName is "auto", "iso8601" should be omitted.
features: [Temporal]
---*/

const date = new Temporal.ZonedDateTime(3661_987_654_321n, "UTC");
const result = date.toString({ calendarName: "auto" });
assert.sameValue(result, "1970-01-01T01:01:01.987654321+00:00[UTC]", `built-in ISO calendar for calendarName = auto`);
