// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.formatRangeToParts
description: >
  ToDateTimeFormattable is called on both arguments before checking if the
  arguments have a different kind.
info: |
  Intl.DateTimeFormat.prototype.formatRangeToParts ( startDate, endDate )

  ...
  4. Let x be ? ToDateTimeFormattable(startDate).
  5. Let y be ? ToDateTimeFormattable(endDate).
  6. Return ? FormatDateTimeRangeToParts(dtf, x, y).

  ToDateTimeFormattable ( value )

  1. If IsTemporalObject(value) is true, return value.
  2. Return ? ToNumber(value).

  FormatDateTimeRangeToParts ( dateTimeFormat, x, y )

  1. Let parts be ? PartitionDateTimeRangePattern(dateTimeFormat, x, y).
  ...

  PartitionDateTimeRangePattern ( dateTimeFormat, x, y )

  1. If IsTemporalObject(x) is true or IsTemporalObject(y) is true, then
    a. If SameTemporalType(x, y) is false, throw a TypeError exception.
  ...
features: [Temporal]
---*/

var callCount = 0;

var invalidDateValue = {
  valueOf() {
    callCount += 1;

    // TimeClip(NaN) throws a RangeError in HandleDateTimeOthers, so if we see
    // a RangeError below, it means the implementation incorrectly calls
    // HandleDateTimeOthers before checking for different argument kinds.
    return NaN;
  }
};

var objects = [
  new Temporal.PlainDate(1970, 1, 1),
  new Temporal.PlainDateTime(1970, 1, 1),
  new Temporal.PlainTime(),
  new Temporal.PlainYearMonth(1970, 1),
  new Temporal.PlainMonthDay(1, 1),
  new Temporal.ZonedDateTime(0n, "UTC"),
  new Temporal.Instant(0n),
];

var dtf = new Intl.DateTimeFormat();

for (var i = 0; i < objects.length; ++i) {
  var object = objects[i];

  assert.sameValue(callCount, i * 2);

  assert.throws(TypeError, function() {
    dtf.formatRangeToParts(invalidDateValue, object);
  });

  assert.sameValue(callCount, i * 2 + 1);

  assert.throws(TypeError, function() {
    dtf.formatRangeToParts(object, invalidDateValue);
  });

  assert.sameValue(callCount, i * 2 + 2);
}
