// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.protoype.total
description: RangeError thrown when unit option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaldurationtotal step 1:
      1. Let _unit_ be ? GetOption(_normalizedOptions_, *"unit"*, « String », « *"year"*, *"years"*, *"month"*, *"months"*, *"week"*, *"weeks"*, *"day"*, *"days"*, *"hour"*, *"hours"*, *"minute"*, *"minutes"*, *"second"*, *"seconds"*, *"millisecond"*, *"milliseconds"*, *"microsecond"*, *"microseconds"*, *"nanosecond"*, *"nanoseconds"* », *undefined*).
    sec-temporal.duration.protoype.total step 5:
      5. Let _unit_ be ? ToTemporalDurationTotalUnit(_options_).
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 1);
assert.throws(RangeError, () => duration.total({ unit: "other string" }));
