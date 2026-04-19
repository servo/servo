// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.reduceright
description: TypedArray instance buffer can be resized during iteration
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, resizable-arraybuffer]
---*/

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(BPE * 3, {maxByteLength: BPE * 3});
  var sample = new TA(buffer);
  var expectedPrevsShrink, expectedNextsShrink, expectedIndicesShrink, expectedArraysShrink;
  var expectedPrevsGrow, expectedNextsGrow, expectedIndicesGrow, expectedArraysGrow;
  var prevs, nexts, indices, arrays, result;

  prevs = [];
  nexts = [];
  indices = [];
  arrays = [];
  result = Array.prototype.reduceRight.call(sample, function(prev, next, index, array) {
    if (prevs.length === 0) {
      try {
        buffer.resize(BPE);
        expectedPrevsShrink = [262, 2];
        expectedNextsShrink = [0, 0];
        expectedIndicesShrink = [2, 0];
        expectedArraysShrink = [sample, sample];
        expectedPrevsGrow = [262];
        expectedNextsGrow = [0];
        expectedIndicesGrow = [0];
        expectedArraysGrow = [sample];
      } catch (_) {
        expectedPrevsShrink = expectedPrevsGrow = [262, 2, 1];
        expectedNextsShrink = expectedNextsGrow = [0, 0, 0];
        expectedIndicesShrink = expectedIndicesGrow = [2, 1, 0];
        expectedArraysShrink = expectedArraysGrow = [sample, sample, sample];
      }
    }

    prevs.push(prev);
    nexts.push(next);
    indices.push(index);
    arrays.push(array);
    return index;
  }, 262);

  assert.compareArray(prevs, expectedPrevsShrink, 'prevs (shrink)');
  assert.compareArray(nexts, expectedNextsShrink, 'nexts (shrink)');
  assert.compareArray(indices, expectedIndicesShrink, 'indices (shrink)');
  assert.compareArray(arrays, expectedArraysShrink, 'arrays (shrink)');
  assert.sameValue(result, 0, 'result (shrink)');

  prevs = [];
  nexts = [];
  indices = [];
  arrays = [];
  result = Array.prototype.reduceRight.call(sample, function(prev, next, index, array) {
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

  assert.compareArray(prevs, expectedPrevsGrow, 'prevs (grow)');
  assert.compareArray(nexts, expectedNextsGrow, 'nexts (grow)');
  assert.compareArray(indices, expectedIndicesGrow, 'indices (grow)');
  assert.compareArray(arrays, expectedArraysGrow, 'arrays (grow)');
  assert.sameValue(result, expectedIndicesGrow[expectedIndicesGrow.length - 1], 'result (grow)');
}, null, ["passthrough"]);
