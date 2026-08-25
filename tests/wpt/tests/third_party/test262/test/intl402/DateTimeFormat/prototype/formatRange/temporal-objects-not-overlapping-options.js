// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: >
  Temporal objects cannot be formatted if there is no overlap between the
  provided options and the data model of the object
features: [Temporal]
---*/

const sampleEpochMs = 1726773817847;  /* 2024-09-19T19:23:37.847Z */
const sampleEpochNs = BigInt(sampleEpochMs) * 1_000_000n;
const legacyDate = new Date(sampleEpochMs);
const instant = new Temporal.Instant(sampleEpochNs);
const plainDateTime = new Temporal.PlainDateTime(2024, 9, 19, 12, 23, 37, 847);
const plainDate = new Temporal.PlainDate(2024, 9, 19);
const plainYearMonth = new Temporal.PlainYearMonth(2024, 9);
const plainMonthDay = new Temporal.PlainMonthDay(9, 19);
const plainTime = new Temporal.PlainTime(12, 23, 37, 847);

const dateStyleFormatter = new Intl.DateTimeFormat(undefined, { dateStyle: "short", calendar: "iso8601", timeZone: "America/Vancouver" });
const dateStyleResult = dateStyleFormatter.formatRange(legacyDate, legacyDate);

assert.sameValue(dateStyleFormatter.formatRange(instant, instant), dateStyleResult, "Instant with dateStyle");
assert.sameValue(dateStyleFormatter.formatRange(plainDateTime, plainDateTime), dateStyleResult, "PlainDateTime with dateStyle");
assert.sameValue(dateStyleFormatter.formatRange(plainDate, plainDate), dateStyleResult, "PlainDate with dateStyle");
assert.notSameValue(dateStyleFormatter.formatRange(plainYearMonth, plainYearMonth), dateStyleResult, "PlainYearMonth with dateStyle should not throw but day is omitted");
assert.notSameValue(dateStyleFormatter.formatRange(plainMonthDay, plainMonthDay), dateStyleResult, "PlainMonthDay with dateStyle should not throw but year is omitted");
assert.throws(TypeError, () => dateStyleFormatter.formatRange(plainTime, plainTime), "no overlap between dateStyle and PlainTime");

const yearFormatter = new Intl.DateTimeFormat(undefined, { year: "numeric", calendar: "iso8601", timeZone: "America/Vancouver" });
const yearResult = yearFormatter.formatRange(legacyDate, legacyDate);

assert.sameValue(yearFormatter.formatRange(instant, instant), yearResult, "Instant with year");
assert.sameValue(yearFormatter.formatRange(plainDateTime, plainDateTime), yearResult, "PlainDateTime with year");
assert.sameValue(yearFormatter.formatRange(plainDate, plainDate), yearResult, "PlainDate with year");
assert.sameValue(yearFormatter.formatRange(plainYearMonth, plainYearMonth), yearResult, "PlainYearMonth with year");
assert.throws(TypeError, () => yearFormatter.formatRange(plainMonthDay, plainMonthDay), "no overlap between year and PlainMonthDay");
assert.throws(TypeError, () => yearFormatter.formatRange(plainTime, plainTime), "no overlap between year and PlainTime");

const dayFormatter = new Intl.DateTimeFormat(undefined, { day: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const dayResult = dayFormatter.formatRange(legacyDate, legacyDate);

assert.sameValue(dayFormatter.formatRange(instant, instant), dayResult, "Instant with day");
assert.sameValue(dayFormatter.formatRange(plainDateTime, plainDateTime), dayResult, "PlainDateTime with day");
assert.sameValue(dayFormatter.formatRange(plainDate, plainDate), dayResult, "PlainDate with day");
assert.throws(TypeError, () => dayFormatter.formatRange(plainYearMonth, plainYearMonth), "no overlap between day and PlainYearMonth");
assert.sameValue(dayFormatter.formatRange(plainMonthDay, plainMonthDay), dayResult, "PlainMonthDay with day");
assert.throws(TypeError, () => dayFormatter.formatRange(plainTime, plainTime), "no overlap between day and PlainTime");

const timeStyleFormatter = new Intl.DateTimeFormat(undefined, { timeStyle: "long", calendar: "iso8601", timeZone: "America/Vancouver" });
const timeStyleResult = timeStyleFormatter.formatRange(legacyDate, legacyDate);

assert.sameValue(timeStyleFormatter.formatRange(instant, instant), timeStyleResult, "Instant with timeStyle");
const timeStylePlainDateTimeResult = timeStyleFormatter.formatRange(plainDateTime, plainDateTime);
assert.notSameValue(timeStylePlainDateTimeResult, timeStyleResult, "PlainDateTime with timeStyle should not throw but time zone is omitted");
assert.throws(TypeError, () => timeStyleFormatter.formatRange(plainDate, plainDate), "no overlap between PlainDate and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.formatRange(plainYearMonth, plainYearMonth), "no overlap between PlainYearMonth and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.formatRange(plainMonthDay, plainMonthDay), "no overlap between PlainMonthDay and timeStyle");
assert.sameValue(timeStyleFormatter.formatRange(plainTime, plainTime), timeStylePlainDateTimeResult, "PlainTime with timeStyle should be the same as PlainDateTime");

const hourFormatter = new Intl.DateTimeFormat(undefined, { hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const hourResult = hourFormatter.formatRange(legacyDate, legacyDate);

assert.sameValue(hourFormatter.formatRange(instant, instant), hourResult, "Instant with hour");
assert.sameValue(hourFormatter.formatRange(plainDateTime, plainDateTime), hourResult, "PlainDateTime with hour");
assert.throws(TypeError, () => hourFormatter.formatRange(plainDate, plainDate), "no overlap between PlainDate and hour");
assert.throws(TypeError, () => hourFormatter.formatRange(plainYearMonth, plainYearMonth), "no overlap between PlainYearMonth and hour");
assert.throws(TypeError, () => hourFormatter.formatRange(plainMonthDay, plainMonthDay), "no overlap between PlainMonthDay and hour");
assert.sameValue(hourFormatter.formatRange(plainTime, plainTime), hourResult, "PlainTime with hour");

const monthFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthResult = monthFormatter.formatRange(legacyDate, legacyDate);
const monthHourFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthHourResult = monthHourFormatter.formatRange(legacyDate, legacyDate);

assert.sameValue(monthHourFormatter.formatRange(instant, instant), monthHourResult, "Instant with month+hour");
assert.sameValue(monthHourFormatter.formatRange(plainDateTime, plainDateTime), monthHourResult, "PlainDateTime with month+hour");
assert.sameValue(monthHourFormatter.formatRange(plainDate, plainDate), monthResult, "PlainDate with month+hour behaves the same as month");
assert.sameValue(monthHourFormatter.formatRange(plainYearMonth, plainYearMonth), monthResult, "PlainYearMonth with month+hour behaves the same as month");
assert.sameValue(monthHourFormatter.formatRange(plainMonthDay, plainMonthDay), monthResult, "PlainMonthDay with month+hour behaves the same as month");
assert.sameValue(monthHourFormatter.formatRange(plainTime, plainTime), hourResult, "PlainTime with month+hour behaves the same as hour");
