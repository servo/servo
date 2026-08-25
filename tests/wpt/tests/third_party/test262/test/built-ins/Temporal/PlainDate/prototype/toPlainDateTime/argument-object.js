// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: Tests for toPlainDateTime() with an object argument.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(2000, 5, 2);
const calendar = { toString() { return "iso8601" } };
const withOverflow = plainDate.toPlainDateTime({ hour: 25, minute: 70, second: 23 });
TemporalHelpers.assertPlainDateTime(withOverflow, 2000, 5, "M05", 2, 23, 59, 23, 0, 0, 0, "with overflow");
assert.sameValue(withOverflow.calendarId, plainDate.calendarId, "with overflow calendar");

const withCalendar = plainDate.toPlainDateTime({ hour: 13, calendar });
TemporalHelpers.assertPlainDateTime(withCalendar, 2000, 5, "M05", 2, 13, 0, 0, 0, 0, 0, "with calendar");
assert.sameValue(withCalendar.calendarId, plainDate.calendarId, "with calendar calendar");
