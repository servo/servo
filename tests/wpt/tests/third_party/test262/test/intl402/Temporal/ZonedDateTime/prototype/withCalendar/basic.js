// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withcalendar
description: Basic functionality of withCalendar
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1572342398_271_986_102n, "-07:00", "gregory");
const result = instance.withCalendar("japanese");
assert.sameValue(result.calendarId, "japanese", "withCalendar() returns a new instance with different calendarId");
