// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.Duration.from handles a property bag if any value is -Infinity
esid: sec-temporal.duration.from
features: [Temporal]
---*/

const fields = ['years', 'months', 'weeks', 'days', 'hours', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds'];

fields.forEach((field) => {
  assert.throws(RangeError, () => Temporal.Duration.from({ [field]: -Infinity }));
});

let calls = 0;
const obj = {
  valueOf() {
    calls++;
    return -Infinity;
  }
};

fields.forEach((field) => {
  calls = 0;
  assert.throws(RangeError, () => Temporal.Duration.from({ [field]: obj }));
  assert.sameValue(calls, 1, "it fails after fetching the primitive value");
});
