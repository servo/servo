// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: Basic tests for withCalendar().
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDate = Temporal.PlainDate.from("1976-11-18");
const calendar = "iso8601";

const stringResult = plainDate.withCalendar("iso8601");
assert.notSameValue(stringResult, plainDate, "string: new object");
TemporalHelpers.assertPlainDate(stringResult, 1976, 11, "M11", 18, "string");
assert.sameValue(stringResult.calendarId, "iso8601", "string: calendar is iso8601");
