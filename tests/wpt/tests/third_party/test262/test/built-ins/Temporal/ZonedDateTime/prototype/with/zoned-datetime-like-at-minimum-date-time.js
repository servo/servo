// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: >
  Throws a RangeError for the minimum date/value with UTC offset.
info: |
  Temporal.ZonedDateTime.prototype.with ( temporalZonedDateTimeLike [ , options ] )
  ...
  21. Let epochNanoseconds be ? InterpretISODateTimeOffset(dateTimeResult.[[Year]],
      dateTimeResult.[[Month]], dateTimeResult.[[Day]], dateTimeResult.[[Hour]],
      dateTimeResult.[[Minute]], dateTimeResult.[[Second]], dateTimeResult.[[Millisecond]],
      dateTimeResult.[[Microsecond]], dateTimeResult.[[Nanosecond]], option, offsetNanoseconds,
      timeZone, disambiguation, offset, match exactly).
  ...
features: [Temporal]
---*/

let zdt = new Temporal.ZonedDateTime(0n, "UTC");

let temporalZonedDateTimeLike = {
  year: -271821,
  month: 4,
  day: 19,
  hour: 1,
  minute: 0,
  second: 0,
  millisecond: 0,
  microsecond: 0,
  nanosecond: 0,
};

assert.throws(RangeError, () => zdt.with(temporalZonedDateTimeLike));
