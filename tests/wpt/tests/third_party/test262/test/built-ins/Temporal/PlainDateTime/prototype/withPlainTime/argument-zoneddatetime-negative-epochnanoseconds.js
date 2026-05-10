// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: A pre-epoch value is handled correctly by the modulo operation in GetISOPartsFromEpoch
info: |
    sec-temporal-getisopartsfromepoch step 1:
      1. Let _remainderNs_ be the mathematical value whose sign is the sign of _epochNanoseconds_ and whose magnitude is abs(_epochNanoseconds_) modulo 10<sup>6</sup>.
    sec-temporal-builtintimezonegetplaindatetimefor step 2:
      2. Let _result_ be ! GetISOPartsFromEpoch(_instant_.[[Nanoseconds]]).
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const datetime = new Temporal.ZonedDateTime(-13849764_999_999_999n, "UTC");

// This code path shows up anywhere we convert an exact time, before the Unix
// epoch, with nonzero microseconds or nanoseconds, into a wall time.

const instance = new Temporal.PlainDateTime(2000, 5, 2, 15, 30, 45, 987, 654, 321);
const result = instance.withPlainTime(datetime);
TemporalHelpers.assertPlainDateTime(result, 2000, 5, "M05", 2, 16, 50, 35, 0, 0, 1);
