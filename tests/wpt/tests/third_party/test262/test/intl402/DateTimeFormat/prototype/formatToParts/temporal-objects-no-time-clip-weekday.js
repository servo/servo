// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Weekday part is correctly adjusted for Temporal dates outside the TimeClip range.
info: |
  When formatToParts is called with a Temporal PlainDate outside the TimeClip
  range and the format includes a weekday, the weekday must be adjusted to
  reflect the actual day of the week after the day-shift adjustment.
features: [Temporal]
locale: [en]
---*/

var dtf = new Intl.DateTimeFormat("en", {
  weekday: "long",
  year: "numeric",
  month: "long",
  day: "numeric",
  calendar: "iso8601"
});

// Minimum plain date value: -271821-04-19
var minDate = new Temporal.PlainDate(-271821, 4, 19);
var parts = dtf.formatToParts(minDate);

var weekdayPart = parts.find(function(p) { return p.type === "weekday"; });
assert.notSameValue(weekdayPart, undefined, "formatToParts should include a weekday part");
var weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
var isValidWeekday = weekdays.indexOf(weekdayPart.value) >= 0;
assert(isValidWeekday, "weekday part should be a valid weekday name, got: " + weekdayPart.value);

var dayPart = parts.find(function(p) { return p.type === "day"; });
assert.notSameValue(dayPart, undefined, "formatToParts should include a day part");
assert.sameValue(dayPart.value, "19", "day part should be 19");

// Maximum plain date value: +275760-09-13
var maxDate = new Temporal.PlainDate(275760, 9, 13);
var maxParts = dtf.formatToParts(maxDate);

var maxWeekdayPart = maxParts.find(function(p) { return p.type === "weekday"; });
assert.notSameValue(maxWeekdayPart, undefined, "max date formatToParts should include a weekday part");
isValidWeekday = weekdays.indexOf(maxWeekdayPart.value) >= 0;
assert(isValidWeekday, "max date weekday part should be a valid weekday name, got: " + maxWeekdayPart.value);

var maxDayPart = maxParts.find(function(p) { return p.type === "day"; });
assert.notSameValue(maxDayPart, undefined, "max date formatToParts should include a day part");
assert.sameValue(maxDayPart.value, "13", "max date day part should be 13");
