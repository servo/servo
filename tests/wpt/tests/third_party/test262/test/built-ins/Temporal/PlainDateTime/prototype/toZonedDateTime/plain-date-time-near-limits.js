// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: >
  Throws a RangeError if the date/time value is outside the instant limits
info: |
  Temporal.PlainDateTime.prototype.toZonedDateTime ( temporalTimeZoneLike [ , options ] )
  ...
  6. Let instant be ? BuiltinTimeZoneGetInstantFor(timeZone, dateTime, disambiguation).
  ...
features: [Temporal]
---*/

// Try to create from the minimum date-time.
{
  let dt = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
  assert.throws(RangeError, () => dt.toZonedDateTime("UTC"));
}
{
  let dt = new Temporal.PlainDateTime(-271821, 4, 19, 1, 0, 0, 0, 0, 0);
  assert.throws(RangeError, () => dt.toZonedDateTime("UTC"));
}

// Try to create from the maximum date-time.
{
  let dt = new Temporal.PlainDateTime(275760, 9, 13, 0, 0, 0, 0, 0, 1);
  assert.throws(RangeError, () => dt.toZonedDateTime("UTC"));
}
{
  let dt = new Temporal.PlainDateTime(275760, 9, 13, 1, 0, 0, 0, 0, 0);
  assert.throws(RangeError, () => dt.toZonedDateTime("UTC"));
}
