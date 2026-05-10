// Copyright 2022 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Return an object if date x is greater than y.
info: |
  Intl.DateTimeFormat.prototype.formatRangeToParts ( startDate , endDate )

  4. Let x be ? ToNumber(startDate).
  5. Let y be ? ToNumber(endDate).
  6. Return ? FormatDateTimeRangeToParts(dtf, x, y).

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

assert.sameValue("object", typeof dtf.formatRangeToParts(x, y));
assert.sameValue("object", typeof dtf.formatRangeToParts(x, x));
assert.sameValue("object", typeof dtf.formatRangeToParts(y, y));
assert.sameValue("object", typeof dtf.formatRangeToParts(y, x));
