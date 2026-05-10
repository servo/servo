// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set values from different instances using the same buffer and same
  constructor when underlying ArrayBuffer has been resized
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, resizable-arraybuffer]
---*/

assert.sameValue(
  typeof ArrayBuffer.prototype.resize,
  'function',
  'implements ArrayBuffer.prototype.resize'
);

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 5});
  var source = new TA(ab);
  var target = new TA(ab);
  var expected = [10, 20, 30, 40];

  source[0] = 10;
  source[1] = 20;
  source[2] = 30;
  source[3] = 40;

  try {
    ab.resize(BPE * 5);
    expected = [10, 20, 30, 40, 0];
  } catch (_) {}

  target.set(source);
  assert(compareArray(target, expected), 'following grow');

  try {
    ab.resize(BPE * 3);
    expected = [10, 20, 30];
  } catch (_) {}

  target.set(source);
  assert(compareArray(target, expected), 'following shrink');
}, null, ["passthrough"]);
