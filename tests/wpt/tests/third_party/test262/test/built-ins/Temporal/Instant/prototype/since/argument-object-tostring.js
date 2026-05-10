// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Object is converted to a string, then to Temporal.Instant
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

const arg = {};
assert.throws(RangeError, () => instance.since(arg), "[object Object] is not a valid ISO string");

arg.toString = function() {
  return "1970-01-01T00:00Z";
};
const result = instance.since(arg);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "result of toString is interpreted as ISO string");
