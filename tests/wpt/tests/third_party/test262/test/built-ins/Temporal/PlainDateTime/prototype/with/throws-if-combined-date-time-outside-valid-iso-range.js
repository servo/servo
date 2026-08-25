// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: >
  Throws if combined date-time outside valid ISO date range.
info: |
  Temporal.PlainDateTime.prototype.with ( temporalDateTimeLike [ , options ] )

  ...
  17. Return ? CreateTemporalDateTime(result, calendar).
features: [Temporal]
---*/

var minDateTime = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
var zeroNanoseconds = {nanosecond: 0};

assert.throws(RangeError, () => minDateTime.with(zeroNanoseconds));
