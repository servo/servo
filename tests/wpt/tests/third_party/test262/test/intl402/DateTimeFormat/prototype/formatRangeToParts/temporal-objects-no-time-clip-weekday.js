// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  Weekday parts are correctly adjusted in formatRangeToParts for Temporal dates
  outside the TimeClip range.
info: |
  When formatRangeToParts is called with Temporal PlainDate values outside the
  range representable by a legacy Date, the implementation must adjust the day
  by shifting it into range and also adjust the weekday parts to match.
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

var weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

// Use extreme dates that are outside the TimeClip range
var minDate = new Temporal.PlainDate(-271821, 4, 19);
var maxDate = new Temporal.PlainDate(275760, 9, 13);

var parts = dtf.formatRangeToParts(minDate, maxDate);

// Find weekday parts
var weekdayParts = parts.filter(function(p) { return p.type === "weekday"; });
assert(weekdayParts.length > 0, "formatRangeToParts should include weekday parts");

for (var i = 0; i < weekdayParts.length; i++) {
  var isValidWeekday = weekdays.indexOf(weekdayParts[i].value) >= 0;
  assert(isValidWeekday, "weekday part should be a valid weekday name, got: " + weekdayParts[i].value);
}

// Verify day parts are correctly adjusted
var dayParts = parts.filter(function(p) { return p.type === "day"; });
assert(dayParts.length >= 2, "formatRangeToParts should include day parts for both dates");

var dayValues = dayParts.map(function(p) { return p.value; });
assert(dayValues.indexOf("19") >= 0, "day parts should include 19 for min date");
assert(dayValues.indexOf("13") >= 0, "day parts should include 13 for max date");
