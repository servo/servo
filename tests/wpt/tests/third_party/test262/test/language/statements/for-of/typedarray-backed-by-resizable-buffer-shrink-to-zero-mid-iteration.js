// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  TypedArrays backed by resizable buffers are iterable with for-of and behave
  correctly when the buffer is shrunk during iteration
features: [resizable-arraybuffer]
includes: [compareArray.js, resizableArrayBufferUtils.js]
---*/

function CreateRab(buffer_byte_length, ctor) {
  const rab = CreateResizableArrayBuffer(buffer_byte_length, 2 * buffer_byte_length);
  let ta_write = new ctor(rab);
  for (let i = 0; i < buffer_byte_length / ctor.BYTES_PER_ELEMENT; ++i) {
    ta_write[i] = MayNeedBigInt(ta_write, i % 128);
  }
  return rab;
}

for (let ctor of ctors) {
  const no_elements = 10;
  const offset = 2;
  const buffer_byte_length = no_elements * ctor.BYTES_PER_ELEMENT;
  const byte_offset = offset * ctor.BYTES_PER_ELEMENT;

  // Create various different styles of TypedArrays with the RAB as the
  // backing store and iterate them.

  // Fixed-length TAs appear detached after shrinking out of bounds.
  let rab = CreateRab(buffer_byte_length, ctor);
  const ta = new ctor(rab, 0, 3);
  assert.throws(TypeError, () => {
    TestIterationAndResize(ta, null, rab, 2, 0);
  });
  rab = CreateRab(buffer_byte_length, ctor);
  const ta_with_offset = new ctor(rab, byte_offset, 3);
  assert.throws(TypeError, () => {
    TestIterationAndResize(ta_with_offset, null, rab, 2, 0);
  });

  // Length-tracking TA with offset 0 doesn't throw, but its length gracefully
  // goes to zero too.
  rab = CreateRab(buffer_byte_length, ctor);
  const length_tracking_ta = new ctor(rab);
  TestIterationAndResize(length_tracking_ta, [
    0,
    1
  ], rab, 2, 0);

  // Length-tracking TA which is resized beyond the offset appars detached.
  rab = CreateRab(buffer_byte_length, ctor);
  const length_tracking_ta_with_offset = new ctor(rab, byte_offset);
  assert.throws(TypeError, () => {
    TestIterationAndResize(length_tracking_ta_with_offset, null, rab, 2, 0);
  });
}
