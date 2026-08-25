// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set values from different instances using the same buffer and same
  constructor. srcBuffer values are cached.
includes: [testTypedArray.js, compareArray.js]
features: [SharedArrayBuffer, TypedArray]
---*/

var int_views = [Int8Array, Uint8Array, Int16Array, Uint16Array, Int32Array, Uint32Array];

testWithTypedArrayConstructors(function(TA) {
  var sample, src, result, sab;

  sab = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab);
  sample[0] = 1;
  sample[1] = 2;
  sample[2] = 3;
  sample[3] = 4;
  src = new TA(sample.buffer, 0, 2);
  result = sample.set(src, 0);
  assert(compareArray(sample, [1, 2, 3, 4]), "offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sab = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab);
  sample[0] = 1;
  sample[1] = 2;
  sample[2] = 3;
  sample[3] = 4;
  src = new TA(sample.buffer, 0, 2);
  result = sample.set(src, 1);
  assert(compareArray(sample, [1, 1, 2, 4]), "offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sab = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab);
  sample[0] = 1;
  sample[1] = 2;
  sample[2] = 3;
  sample[3] = 4;
  src = new TA(sample.buffer, 0, 2);
  result = sample.set(src, 2);
  assert(compareArray(sample, [1, 2, 1, 2]), "offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");
}, int_views);
