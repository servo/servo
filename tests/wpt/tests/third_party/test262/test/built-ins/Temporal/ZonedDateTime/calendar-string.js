// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.constructor
description: A calendar ID is valid input for Calendar
features: [Temporal]
---*/

const arg = "iso8601";

const result = new Temporal.ZonedDateTime(0n, "UTC", arg);
assert.sameValue(result.calendarId, "iso8601", `Calendar created from string "${arg}"`);
