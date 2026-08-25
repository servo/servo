// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
includes: [deepEqual.js]
features: [Temporal, Intl.DateTimeFormat-formatRange]
---*/

// Tolerate implementation variance by expecting consistency without being prescriptive.
// TODO: can we change tests to be less reliant on CLDR formats while still testing that
// Temporal and Intl are behaving as expected?
const usDayPeriodSpace =
  new Intl.DateTimeFormat('en-US', { timeStyle: 'short' })
    .formatRangeToParts(0, 86400)
    .find((part, i, parts) => part.type === 'literal' && parts[i + 1].type === 'dayPeriod')?.value || '';
const usDateRangeSeparator = new Intl.DateTimeFormat('en-US', { dateStyle: 'short' })
  .formatRangeToParts(1 * 86400 * 1000, 366 * 86400 * 1000)
  .find((part) => part.type === 'literal' && part.source === 'shared').value;

const formatter = new Intl.DateTimeFormat('en-US', { timeZone: 'Pacific/Apia' });

const date1 = new Temporal.PlainDate(2021, 8, 4);
const date2 = new Temporal.PlainDate(2021, 8, 5);
const dateResult = formatter.formatRangeToParts(date1, date2);
assert.deepEqual(dateResult, [
  { type: "month", value: "8", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "4", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2021", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "8", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "5", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2021", source: "endRange" },
], "plain dates");

const datetime1 = new Temporal.PlainDateTime(2021, 8, 4, 0, 30, 45, 123, 456, 789);
const datetime2 = new Temporal.PlainDateTime(2021, 8, 4, 23, 30, 45, 123, 456, 789);
const datetimeResult = formatter.formatRangeToParts(datetime1, datetime2);
assert.deepEqual(datetimeResult, [
  { type: "month", value: "8", source: "shared" },
  { type: "literal", value: "/", source: "shared" },
  { type: "day", value: "4", source: "shared" },
  { type: "literal", value: "/", source: "shared" },
  { type: "year", value: "2021", source: "shared" },
  { type: "literal", value: ", ", source: "shared" },
  { type: "hour", value: "12", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "minute", value: "30", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "45", source: "startRange" },
  { type: "literal", value: usDayPeriodSpace, source: "startRange" },
  { type: "dayPeriod", value: "AM", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "hour", value: "11", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "minute", value: "30", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "45", source: "endRange" },
  { type: "literal", value: usDayPeriodSpace, source: "endRange" },
  { type: "dayPeriod", value: "PM", source: "endRange" },
], "plain datetimes");

const monthDay1 = new Temporal.PlainMonthDay(8, 4, "gregory");
const monthDay2 = new Temporal.PlainMonthDay(8, 5, "gregory");
const monthDayResult = formatter.formatRangeToParts(monthDay1, monthDay2);
assert.deepEqual(monthDayResult, [
  { type: "month", value: "8", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "4", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "8", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "5", source: "endRange" },
], "plain month-days");

const time1 = new Temporal.PlainTime(0, 30, 45, 123, 456, 789);
const time2 = new Temporal.PlainTime(23, 30, 45, 123, 456, 789);
const timeResult = formatter.formatRangeToParts(time1, time2);
assert.deepEqual(timeResult, [
  { type: "hour", value: "12", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "minute", value: "30", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "45", source: "startRange" },
  { type: "literal", value: usDayPeriodSpace, source: "startRange" },
  { type: "dayPeriod", value: "AM", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "hour", value: "11", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "minute", value: "30", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "45", source: "endRange" },
  { type: "literal", value: usDayPeriodSpace, source: "endRange" },
  { type: "dayPeriod", value: "PM", source: "endRange" },
], "plain times");

const month1 = new Temporal.PlainYearMonth(2021, 8, "gregory");
const month2 = new Temporal.PlainYearMonth(2021, 9, "gregory");
const monthResult = formatter.formatRangeToParts(month1, month2);
assert.deepEqual(monthResult, [
  { type: "month", value: "8", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2021", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "9", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2021", source: "endRange" },
], "plain year-months");
