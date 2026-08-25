// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withcalendar
description: A calendar ID is valid input for Calendar
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC", "iso8601");

const arg = "iso8601";

const result = instance.withCalendar(arg);
assert.sameValue(result.calendarId, "iso8601", `Calendar created from string "${arg}"`);
