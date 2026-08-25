// Copyright (C) 2017 André Bargull. All rights reserved.
// Copyright (C) 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  The Date constructor is not called to convert the input value.
info: |
  Intl.DateTimeFormat.prototype.formatRange ( startDate , endDate )

  5. Let x be ? ToNumber(startDate).
  6. Let y be ? ToNumber(endDate).
  8. Return ? FormatDateTimeRange(dtf, x, y).

  PartitionDateTimeRangePattern ( dateTimeFormat, x, y )

  1. Let x be TimeClip(x).
  2. If x is NaN, throw a RangeError exception.
  3. Let y be TimeClip(y).
  4. If y is NaN, throw a RangeError exception.
features: [Intl.DateTimeFormat-formatRange]
---*/

const dtf = new Intl.DateTimeFormat();
const dateTimeString = "2017-11-10T14:09:00.000Z";
const date = new Date(dateTimeString);
// |dateTimeString| is valid ISO-8601 style date/time string.
assert.notSameValue(date, NaN);

// ToNumber() will try to parse the string as an integer and yield NaN, rather
// than attempting to parse it like the Date constructor would.
assert.throws(RangeError, function() {
  dtf.formatRange(dateTimeString, date);
});

assert.throws(RangeError, function() {
  dtf.formatRange(date, dateTimeString);
});
