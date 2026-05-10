// Copyright (C) 2017 André Bargull. All rights reserved.
// Copyright (C) 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  TimeClip is applied when calling Intl.DateTimeFormat.prototype.formatRange.
info: |
  PartitionDateTimeRangePattern ( dateTimeFormat, x, y )

  1. Let x be TimeClip(x).
  2. If x is NaN, throw a RangeError exception.
  3. Let y be TimeClip(y).
  4. If y is NaN, throw a RangeError exception.

  TimeClip ( time )
  ...
  2. If abs(time) > 8.64 × 10^15, return NaN.
  ...

includes: [dateConstants.js]
features: [Intl.DateTimeFormat-formatRange]
---*/

const dtf = new Intl.DateTimeFormat();
const date = Date.now();

// Test values near the start of the ECMAScript time range.
assert.throws(RangeError, function() {
  dtf.formatRange(start_of_time - 1, date);
});
assert.throws(RangeError, function() {
  dtf.formatRange(date, start_of_time - 1);
});
assert.sameValue(typeof dtf.formatRange(start_of_time, date), "string");
assert.sameValue(typeof dtf.formatRange(start_of_time + 1, date), "string");

// Test values near the end of the ECMAScript time range.
assert.sameValue(typeof dtf.formatRange(date, end_of_time - 1), "string");
assert.sameValue(typeof dtf.formatRange(date, end_of_time), "string");
assert.throws(RangeError, function() {
  dtf.formatRange(end_of_time + 1, date);
});
assert.throws(RangeError, function() {
  dtf.formatRange(date, end_of_time + 1);
});
