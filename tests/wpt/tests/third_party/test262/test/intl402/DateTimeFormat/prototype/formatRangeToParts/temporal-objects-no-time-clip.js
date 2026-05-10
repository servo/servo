// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: TimeClip is not applied for Temporal plain objects.
info: |
  PartitionDateTimeRangePattern ( _dateTimeFormat_, _x_, _y_ )
    2. Let _xFormatRecord_ be ? ValueFormatRecord(_dateTimeFormat_, _x_).
    3. Let _yFormatRecord_ be ? ValueFormatRecord(_dateTimeFormat_, _y_).
features: [Temporal]
locale: [en]
---*/

// Based on the test in
// DateTime/prototype/format/temporal-objects-no-time-clip.js by André Bargull.

function findPart(parts, expectedType, expectedSource) {
  return parts.find(({ type, source }) => type === expectedType && source === expectedSource).value;
}

// Test with Temporal default calendar "iso8601" and additionally "gregory".
var calendars = ["iso8601", "gregory"];

for (var calendar of calendars) {
  var dtf = new Intl.DateTimeFormat("en", {calendar});

  // Minimum and maximum PlainDate values
  var minDate = new Temporal.PlainDate(-271821, 4, 19, calendar);
  var maxDate = new Temporal.PlainDate(275760, 9, 13, calendar);
  var dateParts = dtf.formatRangeToParts(minDate, maxDate);
  var yearPart = +findPart(dateParts, "year", "startRange");
  assert(yearPart === -271821 || yearPart === 271822, "dateParts includes min year");
  assert.sameValue(+findPart(dateParts, "month", "startRange"), 4, "dateParts includes min month");
  assert.sameValue(+findPart(dateParts, "day", "startRange"), 19, "dateParts includes min day");
  assert.sameValue(+findPart(dateParts, "year", "endRange"), 275760, "dateParts includes max year");
  assert.sameValue(+findPart(dateParts, "month", "endRange"), 9, "dateParts includes max month");
  assert.sameValue(+findPart(dateParts, "day", "endRange"), 13, "dateParts includes max day");

  // Minimum and maximum PlainDateTime values
  var minDateTime = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1, calendar);
  var maxDateTime = new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999, calendar);
  var dateTimeParts = dtf.formatRangeToParts(minDateTime, maxDateTime);
  yearPart = +findPart(dateTimeParts, "year", "startRange");
  assert(yearPart === -271821 || yearPart === 271822, "dateTimeParts includes min year");
  assert.sameValue(+findPart(dateTimeParts, "month", "startRange"), 4, "dateTimeParts includes min month");
  assert.sameValue(+findPart(dateTimeParts, "day", "startRange"), 19, "dateTimeParts includes min day");
  assert.sameValue(+findPart(dateTimeParts, "year", "endRange"), 275760, "dateTimeParts includes max year");
  assert.sameValue(+findPart(dateTimeParts, "month", "endRange"), 9, "dateTimeParts includes max month");
  assert.sameValue(+findPart(dateTimeParts, "day", "endRange"), 13, "dateTimeParts includes max day");

  // Minimum and maximum PlainYearMonth values
  var minYearMonth = new Temporal.PlainYearMonth(-271821, 4, calendar);
  var maxYearMonth = new Temporal.PlainYearMonth(275760, 9, calendar);
  var yearMonthParts = dtf.formatRangeToParts(minYearMonth, maxYearMonth);
  yearPart = +findPart(yearMonthParts, "year", "startRange");
  assert(yearPart === -271821 || yearPart === 271822, "yearMonthParts includes min year");
  assert.sameValue(+findPart(yearMonthParts, "month", "startRange"), 4, "yearMonthParts includes min month");
  assert.sameValue(+findPart(yearMonthParts, "year", "endRange"), 275760, "yearMonthParts includes max year");
  assert.sameValue(+findPart(yearMonthParts, "month", "endRange"), 9, "yearMonthParts includes max month");
}
