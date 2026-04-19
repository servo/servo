// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.equals
description: Object is converted to a string, then to Temporal.Instant
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

const arg = {};
assert.throws(RangeError, () => instance.equals(arg), "[object Object] is not a valid ISO string");

arg.toString = function() {
  return "1970-01-01T00:00Z";
};
const result = instance.equals(arg);
assert.sameValue(result, true, "result of toString is interpreted as ISO string");
