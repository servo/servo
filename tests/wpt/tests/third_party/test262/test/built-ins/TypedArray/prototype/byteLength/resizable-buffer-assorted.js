// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-%typedarray%.prototype.bytelength
description: >
  TypedArray.p.byteLength behaves correctly on assorted kinds of receivers
  backed by resizable buffers
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

const rab = CreateResizableArrayBuffer(40, 80);
for (let ctor of ctors) {
  const ta = new ctor(rab, 0, 3);
  assert.compareArray(ta.buffer, rab);
  assert.sameValue(ta.byteLength, 3 * ctor.BYTES_PER_ELEMENT);
  const empty_ta = new ctor(rab, 0, 0);
  assert.compareArray(empty_ta.buffer, rab);
  assert.sameValue(empty_ta.byteLength, 0);
  const ta_with_offset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 3);
  assert.compareArray(ta_with_offset.buffer, rab);
  assert.sameValue(ta_with_offset.byteLength, 3 * ctor.BYTES_PER_ELEMENT);
  const empty_ta_with_offset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 0);
  assert.compareArray(empty_ta_with_offset.buffer, rab);
  assert.sameValue(empty_ta_with_offset.byteLength, 0);
  const length_tracking_ta = new ctor(rab);
  assert.compareArray(length_tracking_ta.buffer, rab);
  assert.sameValue(length_tracking_ta.byteLength, 40);
  const offset = 8;
  const length_tracking_ta_with_offset = new ctor(rab, offset);
  assert.compareArray(length_tracking_ta_with_offset.buffer, rab);
  assert.sameValue(length_tracking_ta_with_offset.byteLength, 40 - offset);
  const empty_length_tracking_ta_with_offset = new ctor(rab, 40);
  assert.compareArray(empty_length_tracking_ta_with_offset.buffer, rab);
  assert.sameValue(empty_length_tracking_ta_with_offset.byteLength, 0);
}
