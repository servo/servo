// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Object is converted to a string, then to Temporal.Instant
features: [Temporal]
---*/

const epoch = new Temporal.Instant(0n);

const arg = {};
assert.throws(RangeError, () => Temporal.Instant.compare(arg, epoch), "[object Object] is not a valid ISO string (first argument)");
assert.throws(RangeError, () => Temporal.Instant.compare(epoch, arg), "[object Object] is not a valid ISO string (second argument)");

arg.toString = function() {
  return "1970-01-01T00:00Z";
};
const result1 = Temporal.Instant.compare(arg, epoch);
assert.sameValue(result1, 0, "result of toString is interpreted as ISO string (first argument)");
const result2 = Temporal.Instant.compare(epoch, arg);
assert.sameValue(result2, 0, "result of toString is interpreted as ISO string (second argument)");
