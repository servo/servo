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
const dateStyleResult = JSON.stringify(dateStyleFormatter.formatRangeToParts(legacyDate, legacyDate));

assert.sameValue(JSON.stringify(dateStyleFormatter.formatRangeToParts(instant, instant)), dateStyleResult, "Instant with dateStyle");
assert.sameValue(JSON.stringify(dateStyleFormatter.formatRangeToParts(plainDateTime, plainDateTime)), dateStyleResult, "PlainDateTime with dateStyle");
assert.sameValue(JSON.stringify(dateStyleFormatter.formatRangeToParts(plainDate, plainDate)), dateStyleResult, "PlainDate with dateStyle");
assert.notSameValue(JSON.stringify(dateStyleFormatter.formatRangeToParts(plainYearMonth, plainYearMonth)), dateStyleResult, "PlainYearMonth with dateStyle should not throw but day is omitted");
assert.notSameValue(JSON.stringify(dateStyleFormatter.formatRangeToParts(plainMonthDay, plainMonthDay)), dateStyleResult, "PlainMonthDay with dateStyle should not throw but year is omitted");
assert.throws(TypeError, () => dateStyleFormatter.formatRangeToParts(plainTime, plainTime), "no overlap between dateStyle and PlainTime");

const yearFormatter = new Intl.DateTimeFormat(undefined, { year: "numeric", calendar: "iso8601", timeZone: "America/Vancouver" });
const yearResult = JSON.stringify(yearFormatter.formatRangeToParts(legacyDate, legacyDate));

assert.sameValue(JSON.stringify(yearFormatter.formatRangeToParts(instant, instant)), yearResult, "Instant with year");
assert.sameValue(JSON.stringify(yearFormatter.formatRangeToParts(plainDateTime, plainDateTime)), yearResult, "PlainDateTime with year");
assert.sameValue(JSON.stringify(yearFormatter.formatRangeToParts(plainDate, plainDate)), yearResult, "PlainDate with year");
assert.sameValue(JSON.stringify(yearFormatter.formatRangeToParts(plainYearMonth, plainYearMonth)), yearResult, "PlainYearMonth with year");
assert.throws(TypeError, () => yearFormatter.formatRangeToParts(plainMonthDay, plainMonthDay), "no overlap between year and PlainMonthDay");
assert.throws(TypeError, () => yearFormatter.formatRangeToParts(plainTime, plainTime), "no overlap between year and PlainTime");

const dayFormatter = new Intl.DateTimeFormat(undefined, { day: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const dayResult = JSON.stringify(dayFormatter.formatRangeToParts(legacyDate, legacyDate));

assert.sameValue(JSON.stringify(dayFormatter.formatRangeToParts(instant, instant)), dayResult, "Instant with day");
assert.sameValue(JSON.stringify(dayFormatter.formatRangeToParts(plainDateTime, plainDateTime)), dayResult, "PlainDateTime with day");
assert.sameValue(JSON.stringify(dayFormatter.formatRangeToParts(plainDate, plainDate)), dayResult, "PlainDate with day");
assert.throws(TypeError, () => dayFormatter.formatRangeToParts(plainYearMonth, plainYearMonth), "no overlap between day and PlainYearMonth");
assert.sameValue(JSON.stringify(dayFormatter.formatRangeToParts(plainMonthDay, plainMonthDay)), dayResult, "PlainMonthDay with day");
assert.throws(TypeError, () => dayFormatter.formatRangeToParts(plainTime, plainTime), "no overlap between day and PlainTime");

const timeStyleFormatter = new Intl.DateTimeFormat(undefined, { timeStyle: "long", calendar: "iso8601", timeZone: "America/Vancouver" });
const timeStyleResult = JSON.stringify(timeStyleFormatter.formatRangeToParts(legacyDate, legacyDate));

assert.sameValue(JSON.stringify(timeStyleFormatter.formatRangeToParts(instant, instant)), timeStyleResult, "Instant with timeStyle");
const timeStylePlainDateTimeResult = JSON.stringify(timeStyleFormatter.formatRangeToParts(plainDateTime, plainDateTime));
assert.notSameValue(timeStylePlainDateTimeResult, timeStyleResult, "PlainDateTime with timeStyle should not throw but time zone is omitted");
assert.throws(TypeError, () => timeStyleFormatter.formatRangeToParts(plainDate, plainDate), "no overlap between PlainDate and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.formatRangeToParts(plainYearMonth, plainYearMonth), "no overlap between PlainYearMonth and timeStyle");
assert.throws(TypeError, () => timeStyleFormatter.formatRangeToParts(plainMonthDay, plainMonthDay), "no overlap between PlainMonthDay and timeStyle");
assert.sameValue(JSON.stringify(timeStyleFormatter.formatRangeToParts(plainTime, plainTime)), timeStylePlainDateTimeResult, "PlainTime with timeStyle should be the same as PlainDateTime");

const hourFormatter = new Intl.DateTimeFormat(undefined, { hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const hourResult = JSON.stringify(hourFormatter.formatRangeToParts(legacyDate, legacyDate));

assert.sameValue(JSON.stringify(hourFormatter.formatRangeToParts(instant, instant)), hourResult, "Instant with hour");
assert.sameValue(JSON.stringify(hourFormatter.formatRangeToParts(plainDateTime, plainDateTime)), hourResult, "PlainDateTime with hour");
assert.throws(TypeError, () => hourFormatter.formatRangeToParts(plainDate, plainDate), "no overlap between PlainDate and hour");
assert.throws(TypeError, () => hourFormatter.formatRangeToParts(plainYearMonth, plainYearMonth), "no overlap between PlainYearMonth and hour");
assert.throws(TypeError, () => hourFormatter.formatRangeToParts(plainMonthDay, plainMonthDay), "no overlap between PlainMonthDay and hour");
assert.sameValue(JSON.stringify(hourFormatter.formatRangeToParts(plainTime, plainTime)), hourResult, "PlainTime with hour");

const monthFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthResult = JSON.stringify(monthFormatter.formatRangeToParts(legacyDate, legacyDate));
const monthHourFormatter = new Intl.DateTimeFormat(undefined, { month: "2-digit", hour: "2-digit", calendar: "iso8601", timeZone: "America/Vancouver" });
const monthHourResult = JSON.stringify(monthHourFormatter.formatRangeToParts(legacyDate, legacyDate));

assert.sameValue(JSON.stringify(monthHourFormatter.formatRangeToParts(instant, instant)), monthHourResult, "Instant with month+hour");
assert.sameValue(JSON.stringify(monthHourFormatter.formatRangeToParts(plainDateTime, plainDateTime)), monthHourResult, "PlainDateTime with month+hour");
assert.sameValue(JSON.stringify(monthHourFormatter.formatRangeToParts(plainDate, plainDate)), monthResult, "PlainDate with month+hour behaves the same as month");
assert.sameValue(JSON.stringify(monthHourFormatter.formatRangeToParts(plainYearMonth, plainYearMonth)), monthResult, "PlainYearMonth with month+hour behaves the same as month");
assert.sameValue(JSON.stringify(monthHourFormatter.formatRangeToParts(plainMonthDay, plainMonthDay)), monthResult, "PlainMonthDay with month+hour behaves the same as month");
assert.sameValue(JSON.stringify(monthHourFormatter.formatRangeToParts(plainTime, plainTime)), hourResult, "PlainTime with month+hour behaves the same as hour");
