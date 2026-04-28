// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Property bag is correctly converted into PlainDate
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const calendar = "iso8601";
const plainDate = Temporal.PlainDate.from({ year: 1976, month: 11, day: 18, calendar });
TemporalHelpers.assertPlainDate(plainDate, 1976, 11, "M11", 18);
assert.sameValue(plainDate.calendarId, "iso8601", "calendar string is iso8601");
const plainDateImplicitCalendar = Temporal.PlainDate.from({ year: 1976, month: 11, day: 18 });
assert.sameValue(plainDateImplicitCalendar.calendarId, "iso8601", "default calendar is iso8601");
