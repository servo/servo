// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: A PlainYearMonth argument is cloned
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDate = Temporal.PlainDate.from("1976-11-18");
const plainYearMonth = Temporal.PlainYearMonth.from(plainDate);
TemporalHelpers.assertPlainYearMonth(plainYearMonth, 1976, 11, "M11");
assert.sameValue(plainYearMonth.calendarId, "iso8601", "calendar string should be iso8601");
assert.sameValue(plainYearMonth.toString({ calendarName: "always" }), "1976-11-01[u-ca=iso8601]", "iso reference date");
