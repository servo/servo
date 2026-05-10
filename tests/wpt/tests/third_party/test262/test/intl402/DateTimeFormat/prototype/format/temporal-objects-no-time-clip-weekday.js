// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Weekday is correctly adjusted for Temporal dates outside the TimeClip range.
info: |
  When a Temporal PlainDate, PlainDateTime, or PlainYearMonth is formatted with
  a weekday option and the date is outside the range representable by a legacy
  Date (i.e., outside the TimeClip range), the implementation must adjust the
  day by shifting it into range, and must also adjust the weekday to match.
features: [Temporal]
locale: [en]
---*/

// Minimum PlainDate value: -271821-04-19
// Maximum PlainDate value: +275760-09-13
// These are outside the range of legacy Date, so the polyfill must adjust
// the day and weekday when formatting.

var dtf = new Intl.DateTimeFormat("en", {
  weekday: "long",
  year: "numeric",
  month: "long",
  day: "numeric",
  calendar: "iso8601"
});

// Minimum plain date value
var minDate = new Temporal.PlainDate(-271821, 4, 19);
var minResult = dtf.format(minDate);
// The result should contain a weekday name
var weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
var hasWeekday = weekdays.some(function(d) { return minResult.includes(d); });
assert(hasWeekday, "min date formatted with weekday should include a weekday name, got: " + minResult);
assert(minResult.includes("19"), "min date result includes day 19, got: " + minResult);

// Maximum plain date value
var maxDate = new Temporal.PlainDate(275760, 9, 13);
var maxResult = dtf.format(maxDate);
hasWeekday = weekdays.some(function(d) { return maxResult.includes(d); });
assert(hasWeekday, "max date formatted with weekday should include a weekday name, got: " + maxResult);
assert(maxResult.includes("13"), "max date result includes day 13, got: " + maxResult);

// Also test with PlainDateTime
var dtfDateTime = new Intl.DateTimeFormat("en", {
  weekday: "short",
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "numeric",
  minute: "numeric",
  calendar: "iso8601"
});

var minDateTime = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
var minDTResult = dtfDateTime.format(minDateTime);
var shortWeekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
var hasDTWeekday = shortWeekdays.some(function(d) { return minDTResult.includes(d); });
assert(hasDTWeekday, "min datetime formatted with weekday should include a weekday name, got: " + minDTResult);
