// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Type conversions for disambiguation option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaldisambiguation step 1:
      1. Return ? GetOption(_normalizedOptions_, *"disambiguation"*, « String », « *"compatible"*, *"earlier"*, *"later"*, *"reject"* », *"compatible"*).
    sec-temporal.zoneddatetime.protoype.with step 14:
      14. Let _disambiguation_ be ? ToTemporalDisambiguation(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
TemporalHelpers.checkStringOptionWrongType("disambiguation", "compatible",
  (disambiguation) => datetime.with({ hour: 2 }, { disambiguation }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_003_600_987_654_321n, descr),
);
