// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Atomics.notify throws on non-shared integer TypedArrays
features: [ArrayBuffer, Atomics, TypedArray]
---*/

const nonsab = new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4);

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, function() {
  Atomics.notify(new Int16Array(nonsab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Int8Array(nonsab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint32Array(nonsab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint16Array(nonsab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint8Array(nonsab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint8ClampedArray(nonsab), poisoned, poisoned);
});
