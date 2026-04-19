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
const dateStyleResult = dateStyleFormatter.format(legacyDate);

assert.sameValue(dateStyleFormatter.format(instant), dateStyleResult, "Instant with dateStyle");
assert.sameValue(dateStyleFormatter.format(plainDateTime), dateStyleResult, "PlainDateTime with dateStyle");
assert.sameValue(dateStyleFormatter.format(plainDate), dateStyleResult, "PlainDate with dateStyle");
assert.notSameValue(dateStyleFormatter.format(plainYearMonth), dateStyleResult, "PlainYearMonth with dateStyle should not throw but day is omitted");
assert.notSameValue(dateStyleFormatter.format(plainMonthDay), dateStyleResult, "PlainMonthDay with dateStyle should not throw but year is omitted");
assert.throws(TypeError, () => dateStyleFormatter.format(plainTime), "no overlap between dateStyle and PlainTime");

const yearFormatter = new Intl.DateTimeFormat(undefined, { year: "numeric", calendar: "iso8601", timeZone: "America/Vancouver" });
const yearResult = yearFormatter.format(legacyDate);

assert.sameValue(yearFormatter.format(instant), yearResult, "Instant with year");
assert.sameValue(yearFormatter.format(plainDateTime), yearResult, "PlainDateTime with year");
assert.sameValue(yearFormatter.format(plainDate), yearResult, "PlainDate with year");
assert.sameValue(yearFormatter.format(plainYearMonth), yearResult, "PlainYearMonth with year");
assert.throws(TypeError, () => yearFormatter.format(plainMonthDay), "no overlap between year and PlainMonthDay");
assert.throws(TypeError, () => yearFormatter.format(plainTime), "no overlap between year and PlainTime");

const dayFormatter = new Intl.DateTimeFormat(undefined, { day: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const dayResult = dayFormatter.format(legacyDate);

assert.sameValue(dayFormatter.format(instant), dayResult, "Instant with day");
assert.sameValue(dayFormatter.format(plainDateTime), dayResult, "PlainDateTime with day");
assert.sameValue(dayFormatter.format(plainDate), dayResult, "PlainDate with day");
assert.throws(TypeError, () => dayFormatter.format(plainYearMonth), "no overlap between day and PlainYearMonth");
assert.sameValue(dayFormatter.format(plainMonthDay), dayResult, "PlainMonthDay with day");
assert.throws(TypeError, () => dayFormatter.format(plainTime), "no overlap between day and PlainTime");

const timeStyleFormatter = new Intl.DateTimeFormat(undefined, { timeStyle: "long", calendar: "iso8601", timeZone: "America/Vancouver" });
const timeStyleResult = timeStyleFormatter.format(legacyDate);

assert.sameValue(timeStyleFormatter.format(instant), timeStyleResult, "Instant with timeStyle");
const timeStylePlainDateTimeResult = timeStyleFormatter.format(plainDateTime);
assert.notSameValue(timeStylePlainDateTimeResult, timeStyleResult, "PlainDateTime with timeStyle should not throw but time zone is omitted");
assert.throws(TypeError, () => timeStyleFormatter.format(plainDate), "no overlap between PlainDate and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.format(plainYearMonth), "no overlap between PlainYearMonth and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.format(plainMonthDay), "no overlap between PlainMonthDay and timeStyle");
assert.sameValue(timeStyleFormatter.format(plainTime), timeStylePlainDateTimeResult, "PlainTime with timeStyle should be the same as PlainDateTime");

const hourFormatter = new Intl.DateTimeFormat(undefined, { hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const hourResult = hourFormatter.format(legacyDate);

assert.sameValue(hourFormatter.format(instant), hourResult, "Instant with hour");
assert.sameValue(hourFormatter.format(plainDateTime), hourResult, "PlainDateTime with hour");
assert.throws(TypeError, () => hourFormatter.format(plainDate), "no overlap between PlainDate and hour");
assert.throws(TypeError, () => hourFormatter.format(plainYearMonth), "no overlap between PlainYearMonth and hour");
assert.throws(TypeError, () => hourFormatter.format(plainMonthDay), "no overlap between PlainMonthDay and hour");
assert.sameValue(hourFormatter.format(plainTime), hourResult, "PlainTime with hour");

const monthFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthResult = monthFormatter.format(legacyDate);
const monthHourFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthHourResult = monthHourFormatter.format(legacyDate);

assert.sameValue(monthHourFormatter.format(instant), monthHourResult, "Instant with month+hour");
assert.sameValue(monthHourFormatter.format(plainDateTime), monthHourResult, "PlainDateTime with month+hour");
assert.sameValue(monthHourFormatter.format(plainDate), monthResult, "PlainDate with month+hour behaves the same as month");
assert.sameValue(monthHourFormatter.format(plainYearMonth), monthResult, "PlainYearMonth with month+hour behaves the same as month");
assert.sameValue(monthHourFormatter.format(plainMonthDay), monthResult, "PlainMonthDay with month+hour behaves the same as month");
assert.sameValue(monthHourFormatter.format(plainTime), hourResult, "PlainTime with month+hour behaves the same as hour");
