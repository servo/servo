// Copyright 2022 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Return a string if date x is greater than y.
info: |
  Intl.DateTimeFormat.prototype.formatRange ( startDate , endDate )
  
  4. Let x be ? ToNumber(startDate).
  5. Let y be ? ToNumber(endDate).
  6. Return ? FormatDateTimeRange(dtf, x, y).

  PartitionDateTimeRangePattern ( dateTimeFormat, x, y )

  1. Let x be TimeClip(x).
  2. If x is NaN, throw a RangeError exception.
  3. Let y be TimeClip(y).
  4. If y is NaN, throw a RangeError exception.

features: [Intl.DateTimeFormat-formatRange]
---*/

var dtf = new Intl.DateTimeFormat();

var x = new Date();
var y = new Date();
x.setDate(y.getDate() + 1);

assert.sameValue("string", typeof dtf.formatRange(x, y));
assert.sameValue("string", typeof dtf.formatRange(x, x));
assert.sameValue("string", typeof dtf.formatRange(y, y));
assert.sameValue("string", typeof dtf.formatRange(y, x));
