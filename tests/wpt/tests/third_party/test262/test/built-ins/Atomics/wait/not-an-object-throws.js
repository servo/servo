// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.wait
description: >
  Throws a TypeError if typedArray arg is not an Object
info: |
  Atomics.wait( typedArray, index, value, timeout )

  1.Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
    ...
    2. if Type(typedArray) is not Object, throw a TypeError exception
features: [Atomics, Symbol]
---*/

var poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, function() {
  Atomics.wait(null, poisoned, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.wait(undefined, poisoned, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.wait(true, poisoned, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.wait(false, poisoned, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.wait('***string***', poisoned, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.wait(Number.NEGATIVE_INFINITY, poisoned, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.wait(Symbol('***symbol***'), poisoned, poisoned, poisoned);
});
