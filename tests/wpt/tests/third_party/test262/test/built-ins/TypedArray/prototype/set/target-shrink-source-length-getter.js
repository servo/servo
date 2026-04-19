// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set
description: >
  TypedArray.p.set behaves correctly on TypedArrays backed by resizable buffers
  that are shrunk due to the source's length getter.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset

function CreateSourceProxy(length, rab, resizeTo) {
  return new Proxy({}, {
    get(target, prop, receiver) {
      if (prop == 'length') {
        rab.resize(resizeTo);
        return length;
      }
      return true; // Can be converted to both BigInt and Number.
    }
  });
}

// Tests where the length getter returns a non-zero value -> these are nop if
// the TA went OOB.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  fixedLength.set(CreateSourceProxy(1, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  fixedLengthWithOffset.set(CreateSourceProxy(1, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  lengthTracking.set(CreateSourceProxy(1, rab, resizeTo));
  assert.compareArray(ToNumbers(lengthTracking), [
    1,
    2,
    4
  ]);
  assert.compareArray(ToNumbers(new ctor(rab)), [
    1,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  lengthTrackingWithOffset.set(CreateSourceProxy(1, rab, resizeTo));
  assert.compareArray(ToNumbers(lengthTrackingWithOffset), [1]);
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    1
  ]);
}

// Length-tracking TA goes OOB because of the offset.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 1 * ctor.BYTES_PER_ELEMENT;
  lengthTrackingWithOffset.set(CreateSourceProxy(1, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [0]);
}

// Tests where the length getter returns a zero -> these don't throw even if
// the TA went OOB.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  fixedLength.set(CreateSourceProxy(0, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  fixedLengthWithOffset.set(CreateSourceProxy(0, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  lengthTracking.set(CreateSourceProxy(0, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  lengthTrackingWithOffset.set(CreateSourceProxy(0, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4
  ]);
}

// Length-tracking TA goes OOB because of the offset.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 1 * ctor.BYTES_PER_ELEMENT;
  lengthTrackingWithOffset.set(CreateSourceProxy(0, rab, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [0]);
}
