// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Type conversions for unit option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaldurationtotal step 1:
      1. Let _unit_ be ? GetOption(_normalizedOptions_, *"unit"*, « String », « *"year"*, *"years"*, *"month"*, *"months"*, *"week"*, *"weeks"*, *"day"*, *"days"*, *"hour"*, *"hours"*, *"minute"*, *"minutes"*, *"second"*, *"seconds"*, *"millisecond"*, *"milliseconds"*, *"microsecond"*, *"microseconds"*, *"nanosecond"*, *"nanoseconds"* », *undefined*).
    sec-temporal.duration.protoype.total step 5:
      5. Let _unit_ be ? ToTemporalDurationTotalUnit(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 1);
TemporalHelpers.checkStringOptionWrongType("unit", "hour",
  (unit) => duration.total({ unit }),
  (result, descr) => assert.sameValue(result, 24, descr),
);
