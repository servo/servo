// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: >
  Throws a RangeError when ZonedDateTime at minimum instant and an explicit +1h offset.
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

let zdt = new Temporal.ZonedDateTime(-86_40000_00000_00000_00000n, "UTC");

let temporalZonedDateTimeLike = {
  offset: "+01",
};

let options = {
  offset: "use",
};

assert.throws(RangeError, () => zdt.with(temporalZonedDateTimeLike, options));
