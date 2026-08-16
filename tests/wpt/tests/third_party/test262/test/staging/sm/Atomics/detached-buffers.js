// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
features: [Atomics]
---*/

const intArrayConstructors = [
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
];

function badValue(ta) {
  return {
    valueOf() {
      $DETACHBUFFER(ta.buffer);
      return 0;
    }
  };
}

// Atomics.load
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.load(ta, badValue(ta)));
}

// Atomics.store
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.store(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.store(ta, 0, badValue(ta)));
}

// Atomics.compareExchange
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.compareExchange(ta, badValue(ta), 0, 0));
  assert.throws(TypeError, () => Atomics.compareExchange(ta, 0, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.compareExchange(ta, 0, 0, badValue(ta)));
}

// Atomics.exchange
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.exchange(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.exchange(ta, 0, badValue(ta)));
}

// Atomics.add
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.add(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.add(ta, 0, badValue(ta)));
}

// Atomics.sub
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.sub(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.sub(ta, 0, badValue(ta)));
}

// Atomics.and
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.and(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.and(ta, 0, badValue(ta)));
}

// Atomics.or
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.or(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.or(ta, 0, badValue(ta)));
}

// Atomics.xor
for (let TA of intArrayConstructors) {
  let ta = new TA(1);

  assert.throws(TypeError, () => Atomics.xor(ta, badValue(ta), 0));
  assert.throws(TypeError, () => Atomics.xor(ta, 0, badValue(ta)));
}
