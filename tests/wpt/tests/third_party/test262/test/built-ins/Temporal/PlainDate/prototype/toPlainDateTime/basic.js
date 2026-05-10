// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: Basic tests for toPlainDateTime().
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);

const string = date.toPlainDateTime("11:30:23");
TemporalHelpers.assertPlainDateTime(string, 2000, 5, "M05", 2, 11, 30, 23, 0, 0, 0, "string");
assert.sameValue(string.calendarId, date.calendarId, "string calendar");

const optionBag = date.toPlainDateTime({ hour: 11, minute: 30, second: 23 });
TemporalHelpers.assertPlainDateTime(optionBag, 2000, 5, "M05", 2, 11, 30, 23, 0, 0, 0, "option bag");
assert.sameValue(optionBag.calendarId, date.calendarId, "option bag calendar");

const plainTime = date.toPlainDateTime(Temporal.PlainTime.from("11:30:23"));
TemporalHelpers.assertPlainDateTime(plainTime, 2000, 5, "M05", 2, 11, 30, 23, 0, 0, 0, "PlainTime");
assert.sameValue(plainTime.calendarId, date.calendarId, "PlainTime calendar");

const plainDateTime = date.toPlainDateTime(Temporal.PlainDateTime.from("1999-07-14T11:30:23"));
TemporalHelpers.assertPlainDateTime(plainDateTime, 2000, 5, "M05", 2, 11, 30, 23, 0, 0, 0, "PlainDateTime");
assert.sameValue(plainDateTime.calendarId, date.calendarId, "PlainDateTime calendar");

const zonedDateTime = date.toPlainDateTime(Temporal.ZonedDateTime.from("1999-07-14T11:30:23Z[UTC]"));
TemporalHelpers.assertPlainDateTime(zonedDateTime, 2000, 5, "M05", 2, 11, 30, 23, 0, 0, 0, "ZonedDateTime");
assert.sameValue(zonedDateTime.calendarId, date.calendarId, "ZonedDateTime calendar");
