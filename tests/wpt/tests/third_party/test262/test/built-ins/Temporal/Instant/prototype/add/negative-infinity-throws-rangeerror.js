// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.Instant.prototype.add throws a RangeError if any value in a property bag is -Infinity
esid: sec-temporal.instant.prototype.add
features: [Temporal]
---*/

const fields = ["hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];

const instance = Temporal.Instant.fromEpochMilliseconds(10_000);

fields.forEach((field) => {
  assert.throws(RangeError, () => instance.add({ [field]: -Infinity }));
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
  assert.throws(RangeError, () => instance.add({ [field]: obj }));
  assert.sameValue(calls, 1, "it fails after fetching the primitive value");
});
