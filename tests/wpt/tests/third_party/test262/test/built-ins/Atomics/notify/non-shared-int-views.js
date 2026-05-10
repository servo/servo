// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test Atomics.notify on non-shared integer TypedArrays
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const sab = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4);

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, function() {
  Atomics.notify(new Int16Array(sab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Int8Array(sab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint32Array(sab),  poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint16Array(sab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint8Array(sab), poisoned, poisoned);
});

assert.throws(TypeError, function() {
  Atomics.notify(new Uint8ClampedArray(sab), poisoned, poisoned);
});
