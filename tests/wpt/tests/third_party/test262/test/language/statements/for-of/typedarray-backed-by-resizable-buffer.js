// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  TypedArrays backed by resizable buffers are iterable with for-of
features: [resizable-arraybuffer]
includes: [compareArray.js, resizableArrayBufferUtils.js]
---*/

function CollectValues(ta) {
  let values = [];
  for (const value of ta) {
    values.push(Number(value));
  }
  return values;
}
for (let ctor of ctors) {
  const no_elements = 10;
  const offset = 2;
  const buffer_byte_length = no_elements * ctor.BYTES_PER_ELEMENT;
  // We can use the same RAB for all the TAs below, since we won't modify it
  // after writing the initial values.
  const rab = CreateResizableArrayBuffer(buffer_byte_length, 2 * buffer_byte_length);
  const byte_offset = offset * ctor.BYTES_PER_ELEMENT;

  // Write some data into the array.
  let ta_write = new ctor(rab);
  for (let i = 0; i < no_elements; ++i) {
    ta_write[i] = MayNeedBigInt(ta_write, i % 128);
  }

  // Create various different styles of TypedArrays with the RAB as the
  // backing store and iterate them.
  const ta = new ctor(rab, 0, 3);
  assert.compareArray(CollectValues(ta), [
    0,
    1,
    2
  ]);
  const empty_ta = new ctor(rab, 0, 0);
  assert.compareArray(CollectValues(empty_ta), []);
  const ta_with_offset = new ctor(rab, byte_offset, 3);
  assert.compareArray(CollectValues(ta_with_offset), [
    2,
    3,
    4
  ]);
  const empty_ta_with_offset = new ctor(rab, byte_offset, 0);
  assert.compareArray(CollectValues(empty_ta_with_offset), []);
  const length_tracking_ta = new ctor(rab);
  {
    let expected = [];
    for (let i = 0; i < no_elements; ++i) {
      expected.push(i % 128);
    }
    assert.compareArray(CollectValues(length_tracking_ta), expected);
  }
  const length_tracking_ta_with_offset = new ctor(rab, byte_offset);
  {
    let expected = [];
    for (let i = offset; i < no_elements; ++i) {
      expected.push(i % 128);
    }
    assert.compareArray(CollectValues(length_tracking_ta_with_offset), expected);
  }
  const empty_length_tracking_ta_with_offset = new ctor(rab, buffer_byte_length);
  assert.compareArray(CollectValues(empty_length_tracking_ta_with_offset), []);
}
