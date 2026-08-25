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
const dateStyleResult = JSON.stringify(dateStyleFormatter.formatToParts(legacyDate));

assert.sameValue(JSON.stringify(dateStyleFormatter.formatToParts(instant)), dateStyleResult, "Instant with dateStyle");
assert.sameValue(JSON.stringify(dateStyleFormatter.formatToParts(plainDateTime)), dateStyleResult, "PlainDateTime with dateStyle");
assert.sameValue(JSON.stringify(dateStyleFormatter.formatToParts(plainDate)), dateStyleResult, "PlainDate with dateStyle");
assert.notSameValue(JSON.stringify(dateStyleFormatter.formatToParts(plainYearMonth)), dateStyleResult, "PlainYearMonth with dateStyle should not throw but day is omitted");
assert.notSameValue(JSON.stringify(dateStyleFormatter.formatToParts(plainMonthDay)), dateStyleResult, "PlainMonthDay with dateStyle should not throw but year is omitted");
assert.throws(TypeError, () => dateStyleFormatter.formatToParts(plainTime), "no overlap between dateStyle and PlainTime");

const yearFormatter = new Intl.DateTimeFormat(undefined, { year: "numeric", calendar: "iso8601", timeZone: "America/Vancouver" });
const yearResult = JSON.stringify(yearFormatter.formatToParts(legacyDate));

assert.sameValue(JSON.stringify(yearFormatter.formatToParts(instant)), yearResult, "Instant with year");
assert.sameValue(JSON.stringify(yearFormatter.formatToParts(plainDateTime)), yearResult, "PlainDateTime with year");
assert.sameValue(JSON.stringify(yearFormatter.formatToParts(plainDate)), yearResult, "PlainDate with year");
assert.sameValue(JSON.stringify(yearFormatter.formatToParts(plainYearMonth)), yearResult, "PlainYearMonth with year");
assert.throws(TypeError, () => yearFormatter.formatToParts(plainMonthDay), "no overlap between year and PlainMonthDay");
assert.throws(TypeError, () => yearFormatter.formatToParts(plainTime), "no overlap between year and PlainTime");

const dayFormatter = new Intl.DateTimeFormat(undefined, { day: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const dayResult = JSON.stringify(dayFormatter.formatToParts(legacyDate));

assert.sameValue(JSON.stringify(dayFormatter.formatToParts(instant)), dayResult, "Instant with day");
assert.sameValue(JSON.stringify(dayFormatter.formatToParts(plainDateTime)), dayResult, "PlainDateTime with day");
assert.sameValue(JSON.stringify(dayFormatter.formatToParts(plainDate)), dayResult, "PlainDate with day");
assert.throws(TypeError, () => dayFormatter.formatToParts(plainYearMonth), "no overlap between day and PlainYearMonth");
assert.sameValue(JSON.stringify(dayFormatter.formatToParts(plainMonthDay)), dayResult, "PlainMonthDay with day");
assert.throws(TypeError, () => dayFormatter.formatToParts(plainTime), "no overlap between day and PlainTime");

const timeStyleFormatter = new Intl.DateTimeFormat(undefined, { timeStyle: "long", calendar: "iso8601", timeZone: "America/Vancouver" });
const timeStyleResult = JSON.stringify(timeStyleFormatter.formatToParts(legacyDate));

assert.sameValue(JSON.stringify(timeStyleFormatter.formatToParts(instant)), timeStyleResult, "Instant with timeStyle");
const timeStylePlainDateTimeResult = JSON.stringify(timeStyleFormatter.formatToParts(plainDateTime));
assert.notSameValue(timeStylePlainDateTimeResult, timeStyleResult, "PlainDateTime with timeStyle should not throw but time zone is omitted");
assert.throws(TypeError, () => timeStyleFormatter.formatToParts(plainDate), "no overlap between PlainDate and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.formatToParts(plainYearMonth), "no overlap between PlainYearMonth and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.formatToParts(plainMonthDay), "no overlap between PlainMonthDay and timeStyle");
assert.sameValue(JSON.stringify(timeStyleFormatter.formatToParts(plainTime)), timeStylePlainDateTimeResult, "PlainTime with timeStyle should be the same as PlainDateTime");

const hourFormatter = new Intl.DateTimeFormat(undefined, { hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const hourResult = JSON.stringify(hourFormatter.formatToParts(legacyDate));

assert.sameValue(JSON.stringify(hourFormatter.formatToParts(instant)), hourResult, "Instant with hour");
assert.sameValue(JSON.stringify(hourFormatter.formatToParts(plainDateTime)), hourResult, "PlainDateTime with hour");
assert.throws(TypeError, () => hourFormatter.formatToParts(plainDate), "no overlap between PlainDate and hour");
assert.throws(TypeError, () => hourFormatter.formatToParts(plainYearMonth), "no overlap between PlainYearMonth and hour");
assert.throws(TypeError, () => hourFormatter.formatToParts(plainMonthDay), "no overlap between PlainMonthDay and hour");
assert.sameValue(JSON.stringify(hourFormatter.formatToParts(plainTime)), hourResult, "PlainTime with hour");

const monthFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthResult = JSON.stringify(monthFormatter.formatToParts(legacyDate));
const monthHourFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthHourResult = JSON.stringify(monthHourFormatter.formatToParts(legacyDate));

assert.sameValue(JSON.stringify(monthHourFormatter.formatToParts(instant)), monthHourResult, "Instant with month+hour");
assert.sameValue(JSON.stringify(monthHourFormatter.formatToParts(plainDateTime)), monthHourResult, "PlainDateTime with month+hour");
assert.sameValue(JSON.stringify(monthHourFormatter.formatToParts(plainDate)), monthResult, "PlainDate with month+hour behaves the same as month");
assert.sameValue(JSON.stringify(monthHourFormatter.formatToParts(plainYearMonth)), monthResult, "PlainYearMonth with month+hour behaves the same as month");
assert.sameValue(JSON.stringify(monthHourFormatter.formatToParts(plainMonthDay)), monthResult, "PlainMonthDay with month+hour behaves the same as month");
assert.sameValue(JSON.stringify(monthHourFormatter.formatToParts(plainTime)), hourResult, "PlainTime with month+hour behaves the same as hour");
