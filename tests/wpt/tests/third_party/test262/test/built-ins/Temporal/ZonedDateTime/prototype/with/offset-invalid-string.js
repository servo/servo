// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.protoype.with
description: RangeError thrown when offset option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaloffset step 1:
      1. Return ? GetOption(_normalizedOptions_, *"offset"*, « String », « *"prefer"*, *"use"*, *"ignore"*, *"reject"* », _fallback_).
    sec-temporal.zoneddatetime.protoype.with step 15:
      15. Let _offset_ be ? ToTemporalOffset(_options_, *"prefer"*).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
assert.throws(RangeError, () => datetime.with({ hour: 2 }, { offset: "other string" }));
