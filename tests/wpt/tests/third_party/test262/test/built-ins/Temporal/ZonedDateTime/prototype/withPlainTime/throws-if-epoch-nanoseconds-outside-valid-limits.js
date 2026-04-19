// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: >
  Throws a RangeError for values outside the valid limits.
info: |
  Temporal.ZonedDateTime.prototype.withPlainTime ( [ plainTimeLike ] )

  ...
  7. Else,
    ...
    c. Let epochNs be ? GetEpochNanosecondsFor(timeZone, resultISODateTime, compatible).
  ...
features: [Temporal]
---*/

var zdt;

zdt = new Temporal.ZonedDateTime(-864n * 10n**19n, "-01");
assert.throws(RangeError, () => zdt.withPlainTime("00:00"));

zdt = new Temporal.ZonedDateTime(-864n * 10n**19n, "+01");
assert.throws(RangeError, () => zdt.withPlainTime("00:00"));

zdt = new Temporal.ZonedDateTime(864n * 10n**19n, "UTC");
assert.throws(RangeError, () => zdt.withPlainTime("01:00"));
