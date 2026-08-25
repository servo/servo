// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: A calendar ID is valid input for Calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const calendar = "iso8601";

const arg = { year: 2019, monthCode: "M06", calendar };
const result = Temporal.PlainYearMonth.from(arg);
TemporalHelpers.assertPlainYearMonth(result, 2019, 6, "M06", `Calendar created from string "${calendar}"`);
assert.sameValue(result.calendarId, "iso8601", "calendar string is iso8601");
