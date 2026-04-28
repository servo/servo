// Copyright (C) 2011 the Sputnik authors. All rights reserved.
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
---*/

assert.throws(TypeError, function() {
  Object.prototype.isPrototypeOf.call(undefined, {});
});
