// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
  This value is coerced to an object.
info: |
  Array.prototype.sort ( comparefn )

  [...]
  2. Let obj be ? ToObject(this value).
  [...]
  12. Return obj.
features: [Symbol, BigInt]
---*/

assert.throws(TypeError, function() {
  [].sort.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  [].sort.call(null);
}, "null");

assert([].sort.call(false) instanceof Boolean, "boolean");
assert([].sort.call(0) instanceof Number, "number");
assert([].sort.call("") instanceof String, "string");
assert([].sort.call(Symbol()) instanceof Symbol, "symbol");
assert([].sort.call(0n) instanceof BigInt, "bigint");
