// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: |
  The Date constructor is not called to convert the input value.
info: >
  12.4.4 Intl.DateTimeFormat.prototype.formatToParts ( date )

  ...
  4. If date is undefined, then
    ...
  5. Else,
    a. Let x be ? ToNumber(date).
  5. Return ? FormatDateTimeToParts(dtf, x).

  12.1.6 PartitionDateTimePattern ( dateTimeFormat, x )

  1. Let x be TimeClip(x).
  2. If x is NaN, throw a RangeError exception.
  3. ...
---*/

var dtf = new Intl.DateTimeFormat();

var dateTimeString = "2017-11-10T14:09:00.000Z";

// |dateTimeString| is valid ISO-8601 style date/time string.
assert.notSameValue(new Date(dateTimeString), NaN);

// Ensure string input values are not converted to time values by calling the
// Date constructor.
assert.throws(RangeError, function() {
    dtf.formatToParts(dateTimeString);
});
