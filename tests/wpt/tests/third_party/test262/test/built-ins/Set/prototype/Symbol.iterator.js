// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set.prototype-@@iterator
description: Initial state of the Symbol.iterator property
info: |
    The initial value of the @@iterator property is the same function object as
    the initial value of the values property.

    Per ES6 section 17, the method should exist on the Set prototype, and it
    should be writable and configurable, but not enumerable.
includes: [propertyHelper.js]
features: [Symbol.iterator]
---*/

assert.sameValue(Set.prototype[Symbol.iterator], Set.prototype.values);
verifyProperty(Set.prototype, Symbol.iterator, {
  writable: true,
  enumerable: false,
  configurable: true,
});
