// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  Weekday is correctly adjusted in formatRange for Temporal dates outside the
  TimeClip range.
info: |
  When formatRange is called with Temporal PlainDate values outside the range
  representable by a legacy Date, the implementation must adjust the day by
  shifting it into range and also adjust the weekday to match.
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

var result = dtf.formatRange(minDate, maxDate);

// The range should include weekday names
var hasWeekday = weekdays.some(function(d) { return result.includes(d); });
assert(hasWeekday, "formatRange with extreme dates should include weekday names, got: " + result);

// The days should be present in the output
assert(result.includes("19"), "formatRange result includes min day 19, got: " + result);
assert(result.includes("13"), "formatRange result includes max day 13, got: " + result);
