// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.map
description: >
  Integer indexed values changed during iteration
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )
includes: [testTypedArray.js]
features: [BigInt, Reflect.set, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n, 44n]));
  var newVal = 0n;

  sample.map(function(val, i) {
    if (i > 0) {
      assert.sameValue(
        sample[i - 1], newVal - 1n,
        "get the changed value during the loop"
      );
      assert.sameValue(
        Reflect.set(sample, 0, 7n),
        true,
        "re-set a value for sample[0]"
      );
    }
    assert.sameValue(
      Reflect.set(sample, i, newVal),
      true,
      "set value during iteration"
    );

    newVal++;
    return 0n;
  });

  assert.sameValue(sample[0], 7n, "changed values after iteration [0] == 7");
  assert.sameValue(sample[1], 1n, "changed values after iteration [1] == 1");
  assert.sameValue(sample[2], 2n, "changed values after iteration [2] == 2");
}, null, null, ["immutable"]);
