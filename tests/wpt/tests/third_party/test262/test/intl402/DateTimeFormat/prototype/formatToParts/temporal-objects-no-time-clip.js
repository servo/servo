// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: TimeClip is not applied for Temporal plain objects.
info: |
  PartitionDateTimePattern ( _dateTimeFormat_, _x_ )
    1. Let _formatRecord_ be ? ValueFormatRecord(_dateTimeFormat_, _x_).
features: [Temporal]
locale: [en]
---*/

// Based on the test in
// DateTime/prototype/format/temporal-objects-no-time-clip.js by André Bargull.

function findPart(parts, expectedType) {
  return parts.find(({ type }) => type === expectedType).value;
}

// Test with Temporal default calendar "iso8601" and additionally "gregory".
var calendars = ["iso8601", "gregory"];

for (var calendar of calendars) {
  var dtf = new Intl.DateTimeFormat("en", {calendar});

  // Minimum plain date value.
  var minDate = dtf.formatToParts(new Temporal.PlainDate(-271821, 4, 19, calendar));
  let yearPart = +findPart(minDate, "year");
  assert(yearPart === -271821 || yearPart === 271822, "minDate includes year");
  assert.sameValue(+findPart(minDate, "month"), 4, "minDate includes month");
  assert.sameValue(+findPart(minDate, "day"), 19, "minDate includes day");

  // Maximum plain date value.
  var maxDate = dtf.formatToParts(new Temporal.PlainDate(275760, 9, 13, calendar));
  assert.sameValue(+findPart(maxDate, "year"), 275760, "maxDate includes year");
  assert.sameValue(+findPart(maxDate, "month"), 9, "maxDate includes month");
  assert.sameValue(+findPart(maxDate, "day"), 13, "maxDate includes day");

  // Minimum plain date-time value.
  var minDateTime = dtf.formatToParts(new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1, calendar));
  yearPart = +findPart(minDateTime, "year");
  assert(yearPart === -271821 || yearPart === 271822, "minDateTime includes year");
  assert.sameValue(+findPart(minDateTime, "month"), 4, "minDateTime includes month");
  assert.sameValue(+findPart(minDateTime, "day"), 19, "minDateTime includes day");

  // Maximum plain date-time value.
  var maxDateTime = dtf.formatToParts(new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999, calendar));
  assert.sameValue(+findPart(maxDateTime, "year"), 275760, "maxDateTime includes year");
  assert.sameValue(+findPart(maxDateTime, "month"), 9, "maxDateTime includes month");
  assert.sameValue(+findPart(maxDateTime, "day"), 13, "maxDateTime includes day");

  // Minimum plain year-month value.
  var minYearMonth = dtf.formatToParts(new Temporal.PlainYearMonth(-271821, 4, calendar));
  yearPart = +findPart(minYearMonth, "year");
  assert(yearPart === -271821 || yearPart === 271822, "minYearMonth includes year");
  assert.sameValue(+findPart(minYearMonth, "month"), 4, "minYearMonth includes month");

  // Maximum plain year-month value.
  var maxYearMonth = dtf.formatToParts(new Temporal.PlainYearMonth(275760, 9, calendar));
  assert.sameValue(+findPart(maxYearMonth, "year"), 275760, "maxYearMonth includes year");
  assert.sameValue(+findPart(maxYearMonth, "month"), 9, "maxYearMonth includes month");
}
