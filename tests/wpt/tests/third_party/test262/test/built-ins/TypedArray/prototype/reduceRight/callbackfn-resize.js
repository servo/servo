// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: Instance buffer can be resized during iteration
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, resizable-arraybuffer]
---*/

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(BPE * 3, {maxByteLength: BPE * 3});
  var sample = new TA(buffer);
  var secondNext, expectedPrevs, expectedNexts, expectedIndices, expectedArrays;
  var prevs, nexts, indices, arrays, result;

  prevs = [];
  nexts = [];
  indices = [];
  arrays = [];
  result = sample.reduceRight(function(prev, next, index, array) {
    if (prevs.length === 0) {
      try {
        buffer.resize(BPE);
        secondNext = undefined;
        expectedPrevs = [262];
        expectedNexts = [0];
        expectedIndices = [0];
        expectedArrays = [sample];
      } catch (_) {
        secondNext = 0;
        expectedPrevs = [262, 2, 1];
        expectedNexts = [0, 0, 0];
        expectedIndices = [2, 1, 0];
        expectedArrays = [sample, sample, sample];
      }
    }

    prevs.push(prev);
    nexts.push(next);
    indices.push(index);
    arrays.push(array);
    return index;
  }, 262);

  assert.compareArray(prevs, [262, 2, 1], 'prevs (shrink)');
  assert.compareArray(nexts, [0, secondNext, 0], 'nexts (shrink)');
  assert.compareArray(indices, [2, 1, 0], 'indices (shrink)');
  assert.compareArray(arrays, [sample, sample, sample], 'arrays (shrink)');
  assert.sameValue(result, 0, 'result (shrink)');

  prevs = [];
  nexts = [];
  indices = [];
  arrays = [];
  result = sample.reduceRight(function(prev, next, index, array) {
    if (prevs.length === 0) {
      try {
        buffer.resize(3 * BPE);
      } catch (_) {}
    }

    prevs.push(prev);
    nexts.push(next);
    indices.push(index);
    arrays.push(array);
    return index;
  }, 262);

  assert.compareArray(prevs, expectedPrevs, 'prevs (grow)');
  assert.compareArray(nexts, expectedNexts, 'nexts (grow)');
  assert.compareArray(indices, expectedIndices, 'indices (grow)');
  assert.compareArray(arrays, expectedArrays, 'arrays (grow)');
  assert.sameValue(result, expectedIndices[expectedIndices.length - 1], 'result (grow)');
}, null, ["passthrough"]);
