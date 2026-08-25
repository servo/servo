// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: |
  TimeClip is applied when calling Intl.DateTimeFormat.prototype.formatToParts.
info: >
  12.1.6 PartitionDateTimePattern ( dateTimeFormat, x )

  1. Let x be TimeClip(x).
  2. If x is NaN, throw a RangeError exception.
  3. ...

  20.3.1.15 TimeClip ( time )
  ...
  2. If abs(time) > 8.64 × 10^15, return NaN.
  ...

includes: [dateConstants.js]
---*/

var dtf = new Intl.DateTimeFormat();

// Test values near the start of the ECMAScript time range.
assert.throws(RangeError, function() {
    dtf.formatToParts(start_of_time - 1);
});
assert.sameValue(typeof dtf.formatToParts(start_of_time), "object");
assert.sameValue(typeof dtf.formatToParts(start_of_time + 1), "object");

// Test values near the end of the ECMAScript time range.
assert.sameValue(typeof dtf.formatToParts(end_of_time - 1), "object");
assert.sameValue(typeof dtf.formatToParts(end_of_time), "object");
assert.throws(RangeError, function() {
    dtf.formatToParts(end_of_time + 1);
});
