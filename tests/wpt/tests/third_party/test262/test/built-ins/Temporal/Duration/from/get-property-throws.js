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
  4. Let days be ? Get(temporalDurationLike, "days").
  ...
  6. Let hours be ? Get(temporalDurationLike, "hours").
  ...
  8. Let microseconds be ? Get(temporalDurationLike, "microseconds").
  ...
  10. Let milliseconds be ? Get(temporalDurationLike, "milliseconds").
  11....
  12. Let minutes be ? Get(temporalDurationLike, "minutes").
  ...
  14. Let months be ? Get(temporalDurationLike, "months").
  ...
  16. Let nanoseconds be ? Get(temporalDurationLike, "nanoseconds").
  ...
  18. Let seconds be ? Get(temporalDurationLike, "seconds").
  ...
  20. Let weeks be ? Get(temporalDurationLike, "weeks").
  ...
  22. Let years be ? Get(temporalDurationLike, "years").
  ...

  ToIntegerIfIntegral ( argument )

  1. Let number be ? ToNumber(argument).
  ...
features: [Temporal]
---*/

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
  var item = {
    get [name]() {
      throw new Test262Error();
    }
  };
  assert.throws(
    Test262Error,
    () => Temporal.Duration.from(item),
    `name = ${name}`
  );
}
