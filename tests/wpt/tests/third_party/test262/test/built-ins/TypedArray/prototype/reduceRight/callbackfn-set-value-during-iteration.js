// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Integer indexed values changed during iteration
info: |
  22.2.3.21 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduceRight is a distinct function that implements the
  same algorithm as Array.prototype.reduceRight as defined in 22.1.3.20 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.20 Array.prototype.reduceRight ( callbackfn [ , initialValue ] )
includes: [testTypedArray.js]
features: [Reflect.set, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43, 44]));
  var newVal = 0;

  sample.reduceRight(function(acc, val, i) {
    if (i < sample.length - 1) {
      assert.sameValue(
        sample[i + 1], newVal - 1,
        "get the changed value during the loop"
      );
      assert.sameValue(
        Reflect.set(sample, 2, 7),
        true,
        "re-set a value for sample[2]"
      );
    }
    assert.sameValue(
      Reflect.set(sample, i, newVal),
      true,
      "set value during iteration"
    );

    newVal++;
  }, 0);

  assert.sameValue(sample[0], 2, "changed values after iteration [0] == 2");
  assert.sameValue(sample[1], 1, "changed values after iteration [1] == 1");
  assert.sameValue(sample[2], 7, "changed values after iteration [2] == 7");
}, null, null, ["immutable"]);
