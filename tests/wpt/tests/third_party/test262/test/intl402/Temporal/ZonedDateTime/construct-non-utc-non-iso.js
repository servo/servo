// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: >
  Test construction and properties of an instance with non-UTC time zone and
  non-ISO8601 calendar
includes: [temporalHelpers.js]
features: [Temporal, BigInt]
---*/

var epochMillis = Date.UTC(1976, 10, 18, 15, 23, 30, 123);
var epochNanos = BigInt(epochMillis) * 1000000n + 456789n;

const instance = new Temporal.ZonedDateTime(epochNanos, "Europe/Vienna", "gregory");
assert(instance instanceof Temporal.ZonedDateTime, "instanceof is correct");
assert.sameValue(typeof instance, "object", "typeof is correct");

assert.sameValue(
  TemporalHelpers.canonicalizeCalendarEra(instance.calendarId, instance.era),
  TemporalHelpers.canonicalizeCalendarEra(instance.calendarId, "ce"),
  "era"
);
assert.sameValue(instance.eraYear, 1976, "eraYear");
assert.sameValue(instance.year, 1976, "year");
assert.sameValue(instance.month, 11, "month");
assert.sameValue(instance.monthCode, "M11", "monthCode");
assert.sameValue(instance.day, 18, "day");
assert.sameValue(instance.hour, 16, "hour");
assert.sameValue(instance.minute, 23, "minute");
assert.sameValue(instance.second, 30, "second");
assert.sameValue(instance.millisecond, 123, "millisecond");
assert.sameValue(instance.microsecond, 456, "microsecond");
assert.sameValue(instance.nanosecond, 789, "nanosecond");

assert.sameValue(instance.epochMilliseconds, 217178610123, "epochMilliseconds");
assert.sameValue(instance.epochNanoseconds, 217178610123456789n, "epochNanoseconds");

assert.sameValue(instance.dayOfWeek, 4, "dayOfWeek");
assert.sameValue(instance.dayOfYear, 323, "dayOfYear");
assert.sameValue(instance.weekOfYear, undefined, "weekOfYear");
assert.sameValue(instance.yearOfWeek, undefined, "yearOfWeek");
assert.sameValue(instance.daysInWeek, 7, "daysInWeek");
assert.sameValue(instance.daysInMonth, 30, "daysInMonth");
assert.sameValue(instance.daysInYear, 366, "daysInYear");
assert.sameValue(instance.monthsInYear, 12, "monthsInYear");
assert.sameValue(instance.inLeapYear, true, "inLeapYear");

assert.sameValue(instance.offset, "+01:00", "offset");
assert.sameValue(instance.offsetNanoseconds, 3600e9, "offsetNanoseconds");
