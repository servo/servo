// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: >
  Change values during predicate call
info: |
  22.2.3.10 %TypedArray%.prototype.find (predicate [ , thisArg ] )

  %TypedArray%.prototype.find is a distinct function that implements the same
  algorithm as Array.prototype.find as defined in 22.1.3.8 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse.

  ...

  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  4. If thisArg was supplied, let T be thisArg; else let T be undefined.
  5. Let k be 0.
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var arr = [1, 2, 3];
  var sample;
  var result;

  sample = new TA(makeCtorArg(3));
  sample.find(function(val, i) {
    sample[i] = arr[i];

    assert.sameValue(val, 0, "value is not mapped to instance");
  });
  assert(compareArray(sample, arr), "values set during each predicate call");

  sample = new TA(arr);
  result = sample.find(function(val, i) {
    if ( i === 0 ) {
      sample[2] = 7;
    }
    return val === 7;
  });
  assert.sameValue(result, 7, "value found");

  sample = new TA(arr);
  result = sample.find(function(val, i) {
    if ( i === 0 ) {
      sample[2] = 7;
    }
    return val === 3;
  });
  assert.sameValue(result, undefined, "value not found");

  sample = new TA(arr);
  result = sample.find(function(val, i) {
    if ( i > 0 ) {
      sample[0] = 7;
    }
    return val === 7;
  });
  assert.sameValue(result, undefined, "value not found - changed after call");

  sample = new TA(arr);
  result = sample.find(function() {
    sample[0] = 7;
    return true;
  });
  assert.sameValue(result, 1, "find() returns previous found value");
}, null, null, ["immutable"]);
