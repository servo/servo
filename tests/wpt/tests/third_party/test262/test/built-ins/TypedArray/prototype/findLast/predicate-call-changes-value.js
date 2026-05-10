// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Change values during predicate call
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  5. Let k be len - 1.
  6. Repeat, while k ≥ 0,
  ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...

includes: [compareArray.js, testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var arr = [1, 2, 3];
  var sample;
  var result;

  sample = new TA(makeCtorArg(3));
  sample.findLast(function(val, i) {
    sample[i] = arr[i];

    assert.sameValue(val, 0, "value is not mapped to instance");
  });
  assert(compareArray(sample, arr), "values set during each predicate call");

  sample = new TA(arr);
  result = sample.findLast(function(val, i) {
    if ( i === 2 ) {
      sample[0] = 7;
    }
    return val === 7;
  });
  assert.sameValue(result, 7, "value found");

  sample = new TA(arr);
  result = sample.findLast(function(val, i) {
    if ( i === 2 ) {
      sample[0] = 7;
    }
    return val === 1;
  });
  assert.sameValue(result, undefined, "value not found");

  sample = new TA(arr);
  result = sample.findLast(function(val, i) {
    if ( i < 2 ) {
      sample[2] = 7;
    }
    return val === 7;
  });
  assert.sameValue(result, undefined, "value not found - changed after call");

  sample = new TA(arr);
  result = sample.findLast(function() {
    sample[2] = 7;
    return true;
  });
  assert.sameValue(result, 3, "findLast() returns previous found value");
});
