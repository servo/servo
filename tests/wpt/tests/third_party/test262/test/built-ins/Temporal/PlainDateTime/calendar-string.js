// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.constructor
description: A calendar ID is valid input for Calendar
features: [Temporal]
---*/

const arg = "iso8601";

const result = new Temporal.PlainDateTime(2000, 5, 2, 15, 23, 30, 987, 654, 321, arg);
assert.sameValue(result.calendarId, "iso8601", `Calendar created from string "${arg}"`);
