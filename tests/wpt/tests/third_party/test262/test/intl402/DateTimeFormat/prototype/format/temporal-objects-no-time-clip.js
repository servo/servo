// Copyright (C) 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  TimeClip is not applied for Temporal plain objects.
info: |
  HandleDateTimeTemporalDate ( dateTimeFormat, temporalDate )
    ...
    2. Let isoDateTime be CombineISODateAndTimeRecord(temporalDate.[[ISODate]], NoonTimeRecord()).
    3. Let epochNs be GetUTCEpochNanoseconds(isoDateTime).
    ...
    6. Return Value Format Record { [[Format]]: format, [[EpochNanoseconds]]: epochNs, [[IsPlain]]: true  }.

  HandleDateTimeTemporalDateTime ( dateTimeFormat, dateTime )
    ...
    2. Let epochNs be GetUTCEpochNanoseconds(dateTime.[[ISODateTime]]).
    ...
    4. Return Value Format Record { [[Format]]: format, [[EpochNanoseconds]]: epochNs, [[IsPlain]]: true  }.

  HandleDateTimeTemporalYearMonth ( dateTimeFormat, temporalYearMonth )
    ...
    2. Let isoDateTime be CombineISODateAndTimeRecord(temporalYearMonth.[[ISODate]], NoonTimeRecord()).
    3. Let epochNs be GetUTCEpochNanoseconds(isoDateTime).
    ...
    6. Return Value Format Record { [[Format]]: format, [[EpochNanoseconds]]: epochNs, [[IsPlain]]: true  }.

  PartitionDateTimePattern ( dateTimeFormat, x )
    1. Let formatRecord be ? HandleDateTimeValue(dateTimeFormat, x).
    2. Let epochNanoseconds be formatRecord.[[EpochNanoseconds]].
    ...
    6. Let result be FormatDateTimePattern(dateTimeFormat, format, pattern, epochNanoseconds, formatRecord.[[IsPlain]]).
    7. Return result.
features: [Temporal]
locale: [en]
---*/

// Test with Temporal default calendar "iso8601" and additionally "gregory".
var calendars = ["iso8601", "gregory"];

for (var calendar of calendars) {
  var dtf = new Intl.DateTimeFormat("en", {calendar});

  // Minimum plain date value.
  var minDate = dtf.format(new Temporal.PlainDate(-271821, 4, 19, calendar));
  assert(minDate.includes("-271821") || minDate.includes("271822"), "minDate includes year");
  assert(minDate.includes("4"), "minDate includes month");
  assert(minDate.includes("19"), "minDate includes day");

  // Maximum plain date value.
  var maxDate = dtf.format(new Temporal.PlainDate(275760, 9, 13, calendar));
  assert(maxDate.includes("275760"), "maxDate includes year");
  assert(maxDate.includes("9"), "maxDate includes month");
  assert(maxDate.includes("13"), "maxDate includes day");

  // Minimum plain date-time value.
  var minDateTime = dtf.format(new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1, calendar));
  assert(minDateTime.includes("-271821") || minDateTime.includes("271822"), "minDateTime includes year");
  assert(minDateTime.includes("4"), "minDateTime includes month");
  assert(minDateTime.includes("19"), "minDateTime includes day");

  // Maximum plain date-time value.
  var maxDateTime = dtf.format(new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999, calendar));
  assert(maxDateTime.includes("275760"), "maxDateTime includes year");
  assert(maxDateTime.includes("9"), "maxDateTime includes month");
  assert(maxDateTime.includes("13"), "maxDateTime includes day");

  // Minimum plain year-month value.
  var minYearMonth = dtf.format(new Temporal.PlainYearMonth(-271821, 4, calendar));
  assert(minYearMonth.includes("-271821") || minYearMonth.includes("271822"), "minYearMonth includes year");
  assert(minYearMonth.includes("4"), "minYearMonth includes month");

  // Maximum plain year-month value.
  var maxYearMonth = dtf.format(new Temporal.PlainYearMonth(275760, 9, calendar));
  assert(maxYearMonth.includes("275760"), "maxYearMonth includes year");
  assert(maxYearMonth.includes("9"), "maxYearMonth includes month");
}
