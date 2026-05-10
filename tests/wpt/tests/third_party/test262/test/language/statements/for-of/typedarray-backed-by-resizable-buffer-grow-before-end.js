// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  TypedArrays backed by resizable buffers are iterable with for-of and behave
  correctly when the buffer is grown during iteration
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

// We need to recreate the RAB between all TA tests, since we grow it.
for (let ctor of ctors) {
  const no_elements = 10;
  const offset = 2;
  const buffer_byte_length = no_elements * ctor.BYTES_PER_ELEMENT;
  const byte_offset = offset * ctor.BYTES_PER_ELEMENT;

  // Create various different styles of TypedArrays with the RAB as the
  // backing store and iterate them.

  let rab = CreateRab(buffer_byte_length, ctor);
  const length_tracking_ta = new ctor(rab);
  {
    let expected = [];
    for (let i = 0; i < no_elements; ++i) {
      expected.push(i % 128);
    }
    // After resizing, the new memory contains zeros.
    for (let i = 0; i < no_elements; ++i) {
      expected.push(0);
    }
    TestIterationAndResize(length_tracking_ta, expected, rab, no_elements, buffer_byte_length * 2);
  }
  rab = CreateRab(buffer_byte_length, ctor);
  const length_tracking_ta_with_offset = new ctor(rab, byte_offset);
  {
    let expected = [];
    for (let i = offset; i < no_elements; ++i) {
      expected.push(i % 128);
    }
    for (let i = 0; i < no_elements; ++i) {
      expected.push(0);
    }
    TestIterationAndResize(length_tracking_ta_with_offset, expected, rab, no_elements - offset, buffer_byte_length * 2);
  }
}
