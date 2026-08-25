// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: Objects of a subclass are never created as return values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnored(
  Temporal.ZonedDateTime,
  [10n, "UTC"],
  "withPlainTime",
  ["05:43:21.123456789"],
  (result) => {
    assert.sameValue(result.epochNanoseconds, 20601_123_456_789n, "epochNanoseconds result");
    assert.sameValue(result.year, 1970, "year result");
    assert.sameValue(result.month, 1, "month result");
    assert.sameValue(result.day, 1, "day result");
    assert.sameValue(result.hour, 5, "hour result");
    assert.sameValue(result.minute, 43, "minute result");
    assert.sameValue(result.second, 21, "second result");
    assert.sameValue(result.millisecond, 123, "millisecond result");
    assert.sameValue(result.microsecond, 456, "microsecond result");
    assert.sameValue(result.nanosecond, 789, "nanosecond result");
  },
);
