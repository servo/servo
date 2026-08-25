// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  Destructuring assignment on TypedArrays backed by resizable buffer
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);

  // Write some data into the array.
  let ta_write = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    ta_write[i] = MayNeedBigInt(ta_write, i);
  }
  {
    let [a, b, c, d, e] = fixedLength;
    assert.compareArray(ToNumbers([
      a,
      b,
      c,
      d
    ]), [
      0,
      1,
      2,
      3
    ]);
    assert.sameValue(e, undefined);
  }
  {
    let [a, b, c] = fixedLengthWithOffset;
    assert.compareArray(ToNumbers([
      a,
      b
    ]), [
      2,
      3
    ]);
    assert.sameValue(c, undefined);
  }
  {
    let [a, b, c, d, e] = lengthTracking;
    assert.compareArray(ToNumbers([
      a,
      b,
      c,
      d
    ]), [
      0,
      1,
      2,
      3
    ]);
    assert.sameValue(e, undefined);
  }
  {
    let [a, b, c] = lengthTrackingWithOffset;
    assert.compareArray(ToNumbers([
      a,
      b
    ]), [
      2,
      3
    ]);
    assert.sameValue(c, undefined);
  }

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    let [a, b, c] = fixedLength;
  });
  assert.throws(TypeError, () => {
    let [a, b, c] = fixedLengthWithOffset;
  });
  {
    let [a, b, c, d] = lengthTracking;
    assert.compareArray(ToNumbers([
      a,
      b,
      c
    ]), [
      0,
      1,
      2
    ]);
    assert.sameValue(d, undefined);
  }
  {
    let [a, b] = lengthTrackingWithOffset;
    assert.compareArray(ToNumbers([a]), [2]);
    assert.sameValue(b, undefined);
  }

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    let [a, b, c] = fixedLength;
  });
  assert.throws(TypeError, () => {
    let [a, b, c] = fixedLengthWithOffset;
  });
  assert.throws(TypeError, () => {
    let [a, b, c] = lengthTrackingWithOffset;
  });
  {
    let [a, b] = lengthTracking;
    assert.compareArray(ToNumbers([a]), [0]);
    assert.sameValue(b, undefined);
  }

  // Shrink to 0.
  rab.resize(0);
  assert.throws(TypeError, () => {
    let [a, b, c] = fixedLength;
  });
  assert.throws(TypeError, () => {
    let [a, b, c] = fixedLengthWithOffset;
  });
  assert.throws(TypeError, () => {
    let [a, b, c] = lengthTrackingWithOffset;
  });
  {
    let [a] = lengthTracking;
    assert.sameValue(a, undefined);
  }

  // Grow so that all TAs are back in-bounds. The new memory is zeroed.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  {
    let [a, b, c, d, e] = fixedLength;
    assert.compareArray(ToNumbers([
      a,
      b,
      c,
      d
    ]), [
      0,
      0,
      0,
      0
    ]);
    assert.sameValue(e, undefined);
  }
  {
    let [a, b, c] = fixedLengthWithOffset;
    assert.compareArray(ToNumbers([
      a,
      b
    ]), [
      0,
      0
    ]);
    assert.sameValue(c, undefined);
  }
  {
    let [a, b, c, d, e, f, g] = lengthTracking;
    assert.compareArray(ToNumbers([
      a,
      b,
      c,
      d,
      e,
      f
    ]), [
      0,
      0,
      0,
      0,
      0,
      0
    ]);
    assert.sameValue(g, undefined);
  }
  {
    let [a, b, c, d, e] = lengthTrackingWithOffset;
    assert.compareArray(ToNumbers([
      a,
      b,
      c,
      d
    ]), [
      0,
      0,
      0,
      0
    ]);
    assert.sameValue(e, undefined);
  }
}
