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

assert.sameValue(Object.prototype.isPrototypeOf.call(undefined, undefined), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(undefined, null), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(undefined, true), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(undefined, "str"), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(undefined, Symbol("desc")), false);
assert.sameValue(Object.prototype.isPrototypeOf.call(undefined, 3.14), false);
