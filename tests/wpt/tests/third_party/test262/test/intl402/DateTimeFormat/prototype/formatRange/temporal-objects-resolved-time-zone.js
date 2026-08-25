// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
features: [Temporal, Intl.DateTimeFormat-formatRange]
---*/

// Tolerate implementation variance by expecting consistency without being prescriptive.
// TODO: can we change tests to be less reliant on CLDR formats while still testing that
// Temporal and Intl are behaving as expected?
const usDayPeriodSpace =
  new Intl.DateTimeFormat("en-US", { timeStyle: "short" })
    .formatRangeToParts(0, 86400)
    .find((part, i, parts) => part.type === "literal" && parts[i + 1].type === "dayPeriod")?.value || "";
const usDateRangeSeparator = new Intl.DateTimeFormat("en-US", { dateStyle: "short" })
  .formatRangeToParts(1 * 86400 * 1000, 366 * 86400 * 1000)
  .find((part) => part.type === "literal" && part.source === "shared").value;

const formatter = new Intl.DateTimeFormat("en-US", { timeZone: "Pacific/Apia" });

const date1 = new Temporal.PlainDate(2021, 8, 4);
const date2 = new Temporal.PlainDate(2021, 8, 5);
const dateResult = formatter.formatRange(date1, date2);
assert.sameValue(dateResult, `8/4/2021${usDateRangeSeparator}8/5/2021`, "plain dates");

const datetime1 = new Temporal.PlainDateTime(2021, 8, 4, 0, 30, 45, 123, 456, 789);
const datetime2 = new Temporal.PlainDateTime(2021, 8, 4, 23, 30, 45, 123, 456, 789);
const datetimeResult = formatter.formatRange(datetime1, datetime2);
assert.sameValue(
  datetimeResult,
  `8/4/2021, 12:30:45${usDayPeriodSpace}AM${usDateRangeSeparator}11:30:45${usDayPeriodSpace}PM`,
  "plain datetimes"
);

const monthDay1 = new Temporal.PlainMonthDay(8, 4, "gregory");
const monthDay2 = new Temporal.PlainMonthDay(8, 5, "gregory");
const monthDayResult = formatter.formatRange(monthDay1, monthDay2);
assert.sameValue(monthDayResult, `8/4${usDateRangeSeparator}8/5`, "plain month-days");

const time1 = new Temporal.PlainTime(0, 30, 45, 123, 456, 789);
const time2 = new Temporal.PlainTime(23, 30, 45, 123, 456, 789);
const timeResult = formatter.formatRange(time1, time2);
assert.sameValue(
  timeResult,
  `12:30:45${usDayPeriodSpace}AM${usDateRangeSeparator}11:30:45${usDayPeriodSpace}PM`,
  "plain times"
);

const month1 = new Temporal.PlainYearMonth(2021, 8, "gregory");
const month2 = new Temporal.PlainYearMonth(2021, 9, "gregory");
const monthResult = formatter.formatRange(month1, month2);
assert.sameValue(monthResult, `8/2021${usDateRangeSeparator}9/2021`, "plain year-months");
