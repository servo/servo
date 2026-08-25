// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
features: [Temporal]
---*/

// Tolerate implementation variance by expecting consistency without being prescriptive.
// TODO: can we change tests to be less reliant on CLDR formats while still testing that
// Temporal and Intl are behaving as expected?
const usDayPeriodSpace =
  new Intl.DateTimeFormat("en-US", { timeStyle: "short" })
    .formatToParts(0)
    .find((part, i, parts) => part.type === "literal" && parts[i + 1].type === "dayPeriod")?.value || "";

const formatter = new Intl.DateTimeFormat("en-US", { timeZone: "Pacific/Apia" });

const date = new Temporal.PlainDate(2021, 8, 4);
const dateResult = formatter.format(date);
assert.sameValue(dateResult, "8/4/2021", "plain date");

const datetime1 = new Temporal.PlainDateTime(2021, 8, 4, 0, 30, 45, 123, 456, 789);
const datetimeResult1 = formatter.format(datetime1);
assert.sameValue(
  datetimeResult1,
  `8/4/2021, 12:30:45${usDayPeriodSpace}AM`,
  "plain datetime close to beginning of day"
);
const datetime2 = new Temporal.PlainDateTime(2021, 8, 4, 23, 30, 45, 123, 456, 789);
const datetimeResult2 = formatter.format(datetime2);
assert.sameValue(datetimeResult2, `8/4/2021, 11:30:45${usDayPeriodSpace}PM`, "plain datetime close to end of day");

const monthDay = new Temporal.PlainMonthDay(8, 4, "gregory");
const monthDayResult = formatter.format(monthDay);
assert.sameValue(monthDayResult, "8/4", "plain month-day");

const time1 = new Temporal.PlainTime(0, 30, 45, 123, 456, 789);
const timeResult1 = formatter.format(time1);
assert.sameValue(timeResult1, `12:30:45${usDayPeriodSpace}AM`, "plain time close to beginning of day");
const time2 = new Temporal.PlainTime(23, 30, 45, 123, 456, 789);
const timeResult2 = formatter.format(time2);
assert.sameValue(timeResult2, `11:30:45${usDayPeriodSpace}PM`, "plain time close to end of day");

const month = new Temporal.PlainYearMonth(2021, 8, "gregory");
const monthResult = formatter.format(month);
assert.sameValue(monthResult, "8/2021", "plain year-month");
