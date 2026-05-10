// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: Basic tests for the PlainMonthDay constructor.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const leapDay = new Temporal.PlainMonthDay(2, 29);
TemporalHelpers.assertPlainMonthDay(leapDay, "M02", 29, "leap day is supported");
assert.sameValue(leapDay.calendarId, "iso8601", "leap day calendar");

const beforeEpoch = new Temporal.PlainMonthDay(12, 2, "iso8601", 1920);
TemporalHelpers.assertPlainMonthDay(beforeEpoch, "M12", 2, "reference year before epoch", 1920);
assert.sameValue(beforeEpoch.calendarId, "iso8601", "reference year before epoch calendar");

const afterEpoch = new Temporal.PlainMonthDay(1, 7, "iso8601", 1980);
TemporalHelpers.assertPlainMonthDay(afterEpoch, "M01", 7, "reference year after epoch", 1980);
assert.sameValue(afterEpoch.calendarId, "iso8601", "reference year after epoch calendar");
