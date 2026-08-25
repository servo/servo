// Copyright (C) 2017 André Bargull. All rights reserved.
// Copyright (C) 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  TimeClip is applied when calling Intl.DateTimeFormat.prototype.formatRangeToParts.
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
  dtf.formatRangeToParts(start_of_time - 1, date);
});
assert.throws(RangeError, function() {
  dtf.formatRangeToParts(date, start_of_time - 1);
});
assert.sameValue(typeof dtf.formatRangeToParts(start_of_time, date), "object");
assert.sameValue(typeof dtf.formatRangeToParts(start_of_time + 1, date), "object");

// Test values near the end of the ECMAScript time range.
assert.sameValue(typeof dtf.formatRangeToParts(date, end_of_time - 1), "object");
assert.sameValue(typeof dtf.formatRangeToParts(date, end_of_time), "object");
assert.throws(RangeError, function() {
  dtf.formatRangeToParts(end_of_time + 1, date);
});
assert.throws(RangeError, function() {
  dtf.formatRangeToParts(date, end_of_time + 1);
});
