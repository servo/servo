// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Fills all the elements with non numeric values values.
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  ...
  3. If O.[[TypedArrayName]] is "BigUint64Array" or "BigInt64Array",
     let value be ? ToBigInt(value).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  var n = 1n;
  sample.fill({ valueOf() { return n++; } });

  assert.sameValue(n, 2n, "additional unexpected ToBigInt() calls");
  assert.sameValue(sample[0], 1n, "incorrect ToNumber result in index 0");
  assert.sameValue(sample[1], 1n, "incorrect ToNumber result in index 1");
}, null, null, ["immutable"]);

