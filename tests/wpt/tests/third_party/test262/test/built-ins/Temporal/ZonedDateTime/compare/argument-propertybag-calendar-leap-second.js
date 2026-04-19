// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Leap second is a valid ISO string for a calendar in a property bag
features: [Temporal]
---*/

const timeZone = "UTC";
const datetime = new Temporal.ZonedDateTime(217_123_200_000_000_000n, timeZone);
const calendar = "2016-12-31T23:59:60+00:00[UTC]";

const arg = { year: 1976, monthCode: "M11", day: 18, timeZone, calendar };
const result1 = Temporal.ZonedDateTime.compare(arg, datetime);
assert.sameValue(result1, 0, "leap second is a valid ISO string for calendar (first argument)");
const result2 = Temporal.ZonedDateTime.compare(datetime, arg);
assert.sameValue(result2, 0, "leap second is a valid ISO string for calendar (second argument)");
