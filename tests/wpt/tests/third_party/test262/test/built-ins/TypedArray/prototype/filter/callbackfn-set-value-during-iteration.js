// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Integer indexed values changed during iteration
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  9. Repeat, while k < len
    ...
    c. Let selected be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [Reflect.set, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43, 44]));
  var newVal = 0;

  sample.filter(function(val, i) {
    if (i > 0) {
      assert.sameValue(
        sample[i - 1], newVal - 1,
        "get the changed value during the loop"
      );
      assert.sameValue(
        Reflect.set(sample, 0, 7),
        true,
        "re-set a value for sample[0]"
      );
    }
    assert.sameValue(
      Reflect.set(sample, i, newVal),
      true,
      "set value during interaction"
    );

    newVal++;
  });

  assert.sameValue(sample[0], 7, "changed values after interaction [0] == 7");
  assert.sameValue(sample[1], 1, "changed values after interaction [1] == 1");
  assert.sameValue(sample[2], 2, "changed values after interaction [2] == 2");
}, null, null, ["immutable"]);
