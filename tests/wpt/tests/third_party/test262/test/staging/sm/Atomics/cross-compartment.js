// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

const otherGlobal = $262.createRealm().global;

const intArrayConstructors = [
  otherGlobal.Int32Array,
  otherGlobal.Int16Array,
  otherGlobal.Int8Array,
  otherGlobal.Uint32Array,
  otherGlobal.Uint16Array,
  otherGlobal.Uint8Array,
];

// Atomics.load
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 1;

  assert.sameValue(Atomics.load(ta, 0), 1);
}

// Atomics.store
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));

  Atomics.store(ta, 0, 1);

  assert.sameValue(ta[0], 1);
}

// Atomics.compareExchange
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 1;

  let val = Atomics.compareExchange(ta, 0, 1, 2);

  assert.sameValue(val, 1);
  assert.sameValue(ta[0], 2);
}

// Atomics.exchange
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 1;

  let val = Atomics.exchange(ta, 0, 2);

  assert.sameValue(val, 1);
  assert.sameValue(ta[0], 2);
}

// Atomics.add
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 1;

  let val = Atomics.add(ta, 0, 2);

  assert.sameValue(val, 1);
  assert.sameValue(ta[0], 3);
}

// Atomics.sub
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 3;

  let val = Atomics.sub(ta, 0, 2);

  assert.sameValue(val, 3);
  assert.sameValue(ta[0], 1);
}

// Atomics.and
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 3;

  let val = Atomics.and(ta, 0, 1);

  assert.sameValue(val, 3);
  assert.sameValue(ta[0], 1);
}

// Atomics.or
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 2;

  let val = Atomics.or(ta, 0, 1);

  assert.sameValue(val, 2);
  assert.sameValue(ta[0], 3);
}

// Atomics.xor
for (let TA of intArrayConstructors) {
  let ta = new TA(new otherGlobal.SharedArrayBuffer(4));
  ta[0] = 3;

  let val = Atomics.xor(ta, 0, 1);

  assert.sameValue(val, 3);
  assert.sameValue(ta[0], 2);
}

