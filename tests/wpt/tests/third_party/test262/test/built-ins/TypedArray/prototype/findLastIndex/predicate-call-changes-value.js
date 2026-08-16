// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Change values during predicate call
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var arr = [10, 20, 30];
  var sample;
  var result;

  sample = new TA(makeCtorArg(3));
  sample.findLastIndex(function(val, i) {
    sample[i] = arr[i];

    assert.sameValue(val, 0, "value is not mapped to instance");
  });
  assert(compareArray(sample, arr), "values set during each predicate call");

  sample = new TA(arr);
  result = sample.findLastIndex(function(val, i) {
    if ( i === 2 ) {
      sample[0] = 7;
    }
    return val === 7;
  });
  assert.sameValue(result, 0, "value found");

  sample = new TA(arr);
  result = sample.findLastIndex(function(val, i) {
    if ( i === 2 ) {
      sample[0] = 7;
    }
    return val === 10;
  });
  assert.sameValue(result, -1, "value not found");

  sample = new TA(arr);
  result = sample.findLastIndex(function(val, i) {
    if ( i < 2) {
      sample[2] = 7;
    }
    return val === 7;
  });
  assert.sameValue(result, -1, "value not found - changed after call");
}, null, null, ["immutable"]);
