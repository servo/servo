// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.zoneddatetimeiso
description: Functions when time zone argument is omitted
features: [Temporal]
---*/

const zdt = Temporal.Now.zonedDateTimeISO();
assert(zdt instanceof Temporal.ZonedDateTime);
assert.sameValue(zdt.calendarId, "iso8601", "calendar string should be iso8601");
assert.sameValue(zdt.timeZoneId, Temporal.Now.timeZoneId(), "time zone string should be the same as from Now");
