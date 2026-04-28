// Copyright 2021 Google Inc. All rights reserved.
// Copyright 2021 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  When startDate is equal to endDate, the output should be a string equal
  to the output of Intl.DateTimeFormat.prototype.format.
info: |
  Intl.DateTimeFormat.prototype.formatRange ( startDate , endDate )
  
  4. Let x be ? ToNumber(startDate).
  5. Let y be ? ToNumber(endDate).
  6. Return ? FormatDateTimeRange(dtf, x, y).

  PartitionDateTimeRangePattern ( dateTimeFormat, x, y )

  13. If dateFieldsPracticallyEqual is true, then
    a. Let pattern be dateTimeFormat.[[Pattern]].
    b. Let patternParts be PartitionPattern(pattern).
    c. Let result be ? FormatDateTimePattern(dateTimeFormat, patternParts, tm1).
    d. For each r in result do
      i. Set r.[[Source]] to "shared".
    e. Return result.

features: [Intl.DateTimeFormat-formatRange]
locale: [en-US]
---*/

{
  const date = new Date(2019, 7, 10,  1, 2, 3, 234);

  let dtf = new Intl.DateTimeFormat("en", { year: "numeric", month: "short", day: "numeric" });
  assert.sameValue(dtf.formatRange(date, date), dtf.format(date), "same output with date options");

  dtf = new Intl.DateTimeFormat("en", { minute: "numeric", second: "numeric" });
  assert.sameValue(dtf.formatRange(date, date), dtf.format(date), "same output with time options");

  dtf = new Intl.DateTimeFormat("en", { month: "short", day: "numeric", minute: "numeric" });
  assert.sameValue(dtf.formatRange(date, date), dtf.format(date), "same output with date-time options");

  dtf = new Intl.DateTimeFormat("en", { dateStyle: "long", timeStyle: "short" });
  assert.sameValue(dtf.formatRange(date, date), dtf.format(date), "same output with dateStyle/timeStyle");
}
{
  const date1 = new Date(2019, 7, 10,  1, 2, 3, 234);
  const date2 = new Date(2019, 7, 10,  1, 2, 3, 235);

  let dtf = new Intl.DateTimeFormat("en", { year: "numeric", month: "short", day: "numeric" });
  assert.sameValue(dtf.formatRange(date1, date2), dtf.format(date1), "same output with date options");

  dtf = new Intl.DateTimeFormat("en", { minute: "numeric", second: "numeric" });
  assert.sameValue(dtf.formatRange(date1, date2), dtf.format(date1), "same output with time options");

  dtf = new Intl.DateTimeFormat("en", { month: "short", day: "numeric", minute: "numeric" });
  assert.sameValue(dtf.formatRange(date1, date2), dtf.format(date1), "same output with date-time options");

  dtf = new Intl.DateTimeFormat("en", { dateStyle: "long", timeStyle: "short" });
  assert.sameValue(dtf.formatRange(date1, date2), dtf.format(date1), "same output with dateStyle/timeStyle");
}
