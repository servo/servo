// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set
description: >
  TypedArray.p.set behaves correctly on TypedArrays backed by resizable buffers
  that are shrunk mid-iteration due to a Proxy source.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset

// Resizing will happen when we're calling Get for the `resizeAt`:th data
// element, but we haven't yet written it to the target.
function CreateSourceProxy(length, rab, resizeAt, resizeTo) {
  let requestedIndices = [];
  return new Proxy({}, {
    get(target, prop, receiver) {
      if (prop == 'length') {
        return length;
      }
      requestedIndices.push(prop);
      if (requestedIndices.length == resizeAt) {
        rab.resize(resizeTo);
      }
      return true; // Can be converted to both BigInt and Number.
    }
  });
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const resizeAt = 2;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  fixedLength.set(CreateSourceProxy(4, rab, resizeAt, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    1,
    2,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const resizeAt = 2;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  fixedLengthWithOffset.set(CreateSourceProxy(2, rab, resizeAt, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    1
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  const resizeAt = 2;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  lengthTracking.set(CreateSourceProxy(2, rab, resizeAt, resizeTo));
  assert.compareArray(ToNumbers(lengthTracking), [
    1,
    1,
    4
  ]);
  assert.compareArray(ToNumbers(new ctor(rab)), [
    1,
    1,
    4
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const resizeAt = 2;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  lengthTrackingWithOffset.set(CreateSourceProxy(2, rab, resizeAt, resizeTo));
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
  const resizeAt = 1;
  const resizeTo = 1 * ctor.BYTES_PER_ELEMENT;
  lengthTrackingWithOffset.set(CreateSourceProxy(2, rab, resizeAt, resizeTo));
  assert.compareArray(ToNumbers(new ctor(rab)), [0]);
}
