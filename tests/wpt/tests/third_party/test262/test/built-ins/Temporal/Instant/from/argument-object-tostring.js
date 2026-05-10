// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Object is converted to a string, then to Temporal.Instant
features: [Temporal]
---*/

const arg = {};
assert.throws(RangeError, () => Temporal.Instant.from(arg), "[object Object] is not a valid ISO string");

const temporalInstant = Temporal.Instant;
assert.throws(RangeError, () => Temporal.Instant.from(temporalInstant), "Temporal.Instant object is not a valid ISO string");

arg.toString = function() {
  return "1970-01-01T00:00Z";
};
const result = Temporal.Instant.from(arg);
assert.sameValue(result.epochNanoseconds, 0n, "result of toString is interpreted as ISO string");
