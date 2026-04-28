// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: >
  ToNumber conversion throws.
info: |
  Temporal.Duration.from ( item )

  1. Return ? ToTemporalDuration(item).

  ToTemporalDuration ( item )

  ...
  4. Let partial be ? ToTemporalPartialDurationRecord(item).
  ...

  ToTemporalPartialDurationRecord ( temporalDurationLike )

  ...
  5. If days is not undefined, set result.[[Days]] to ? ToIntegerIfIntegral(days).
  ...
  7. If hours is not undefined, set result.[[Hours]] to ? ToIntegerIfIntegral(hours).
  ...
  9. If microseconds is not undefined, set result.[[Microseconds]] to ? ToIntegerIfIntegral(microseconds).
  ...
  11. If milliseconds is not undefined, set result.[[Milliseconds]] to ? ToIntegerIfIntegral(milliseconds).
  ...
  13. If minutes is not undefined, set result.[[Minutes]] to ? ToIntegerIfIntegral(minutes).
  ...
  15. If months is not undefined, set result.[[Months]] to ? ToIntegerIfIntegral(months).
  ...
  17. If nanoseconds is not undefined, set result.[[Nanoseconds]] to ? ToIntegerIfIntegral(nanoseconds).
  ...
  19. If seconds is not undefined, set result.[[Seconds]] to ? ToIntegerIfIntegral(seconds).
  ...
  21. If weeks is not undefined, set result.[[Weeks]] to ? ToIntegerIfIntegral(weeks).
  ...
  23. If years is not undefined, set result.[[Years]] to ? ToIntegerIfIntegral(years).
  ...

  ToIntegerIfIntegral ( argument )

  1. Let number be ? ToNumber(argument).
  ...
features: [Temporal]
---*/

for (var invalid of [Symbol(), 0n]) {
  for (var name of [
    "years",
    "months",
    "weeks",
    "days",
    "hours",
    "minutes",
    "seconds",
    "milliseconds",
    "microseconds",
    "nanoseconds",
  ]) {
    var item = {[name]: invalid};
    assert.throws(
      TypeError,
      () => Temporal.Duration.from(item),
      `name = ${name}, value = ${String(invalid)}`
    );
  }
}
