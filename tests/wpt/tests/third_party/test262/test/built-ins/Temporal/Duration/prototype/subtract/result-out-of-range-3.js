// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: >
  TemporalDurationFromInternal throws a RangeError when the result is too large.
info: |
  TemporalDurationFromInternal ( internalDuration, largestUnit )

  ...
  9. Else if largestUnit is microsecond, then
    a. Set microseconds to floor(nanoseconds / 1000).
    b. Set nanoseconds to nanoseconds modulo 1000.
  ...
  12. Return ? CreateTemporalDuration(internalDuration.[[Date]].[[Years]],
      internalDuration.[[Date]].[[Months]], internalDuration.[[Date]].[[Weeks]],
      internalDuration.[[Date]].[[Days]] + days × sign, hours × sign, minutes × sign,
      seconds × sign, milliseconds × sign, microseconds × sign, nanoseconds × sign).
features: [Temporal]
---*/

var one = Temporal.Duration.from({nanoseconds: -9.007199254740991e+24});
var two = Temporal.Duration.from({microseconds: 1_000_000});

// TemporalDurationFromInternal is called with:
//   internalDuration.[[Time]] = -9,007,199,254,740,991,926,258,176.
//
// |microseconds| is -9,007,199,254,740,991,926,258, which rounds up to
// -9,007,199,254,740,992,000,000. This exceeds the maximum allowed duration.
assert.throws(RangeError, () => one.subtract(two));
