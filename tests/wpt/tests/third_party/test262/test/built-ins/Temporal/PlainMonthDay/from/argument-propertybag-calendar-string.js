// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: A calendar ID is valid input for Calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const calendar = "iso8601";

const arg = { monthCode: "M11", day: 18, calendar };
const result = Temporal.PlainMonthDay.from(arg);
TemporalHelpers.assertPlainMonthDay(result, "M11", 18, `Calendar created from string "${calendar}"`);
assert.sameValue(result.calendarId, "iso8601", "calendar string is iso8601");
