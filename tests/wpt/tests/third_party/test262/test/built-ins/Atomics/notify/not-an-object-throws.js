// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.notify
description: >
  Throws a TypeError if typedArray arg is not an Object
info: |
  Atomics.notify( typedArray, index, count )

  1.Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
    ...
    2. if Type(typedArray) is not Object, throw a TypeError exception
features: [Atomics, Symbol]
---*/

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, function() {
  Atomics.notify(null, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(undefined, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(true, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(false, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify('***string***', poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(Number.NEGATIVE_INFINITY, poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(Symbol('***symbol***'), poisoned, poisoned);
});
