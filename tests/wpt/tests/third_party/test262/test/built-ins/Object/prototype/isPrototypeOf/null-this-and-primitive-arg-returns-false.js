// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.isprototypeof
description: >
  The ordering of steps 1 and 2 preserves the behaviour specified by previous
  editions of this specification for the case where V is not an object and
  the this value is undefined or null.
info: |
  Object.prototype.isPrototypeOf ( V )

  1. If Type(V) is not Object, return false.
  2. Let O be ? ToObject(this value).
features: [Symbol]
---*/

assert.sameValue(Object.prototype.isPrototypeOf.call(null, undefined), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(null, null), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(null, false), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(null, ""), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(null, Symbol()), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(null, 10), false);
