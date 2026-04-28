// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Type conversions for offset option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloffset step 1:
      1. Return ? GetOption(_normalizedOptions_, *"offset"*, « String », « *"prefer"*, *"use"*, *"ignore"*, *"reject"* », _fallback_).
    sec-temporal.zoneddatetime.protoype.with step 15:
      15. Let _offset_ be ? ToTemporalOffset(_options_, *"prefer"*).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
TemporalHelpers.checkStringOptionWrongType("offset", "prefer",
  (offset) => datetime.with({ hour: 2 }, { offset }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_003_600_987_654_321n, descr),
);
