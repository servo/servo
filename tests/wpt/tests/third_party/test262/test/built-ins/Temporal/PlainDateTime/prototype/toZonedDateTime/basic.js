// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Straightforward case of using UTC
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(2020, 1, 1, 0, 0);
const zdt = dt.toZonedDateTime("UTC");

assert.sameValue(zdt.epochNanoseconds, 1577836800000000000n, "nanoseconds");
assert.sameValue(zdt.calendarId, "iso8601", "calendar");
assert.sameValue(zdt.timeZoneId, "UTC", "timezone");
