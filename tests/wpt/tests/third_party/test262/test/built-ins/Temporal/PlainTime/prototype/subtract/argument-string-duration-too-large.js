// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.subtract
description: >
  ParseTemporalDurationString throws a RangeError when the result is too large.
features: [Temporal]
---*/

// Number string too long to be representable as a Number value.
var ones = "1".repeat(1000);
assert.sameValue(Number(ones), Infinity);

var time = new Temporal.PlainTime();
var str = "PT" + ones + "S";

assert.throws(RangeError, () => time.subtract(str));
