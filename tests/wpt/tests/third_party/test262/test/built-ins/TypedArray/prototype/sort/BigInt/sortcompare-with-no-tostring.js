// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: TypedArrays sort does not cast values to String
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  When the TypedArray SortCompare abstract operation is called with two
  arguments x and y, the following steps are taken:

  ...
  2. If the argument comparefn is not undefined, then
    a. Let v be ? Call(comparefn, undefined, « x, y »).
    ...
  ...
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

var toStringCalled = false;
BigInt.prototype.toString = function() {
  toStringCalled = true;
}

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([20n, 100n, 3n]));
  var result = sample.sort();
  assert.sameValue(toStringCalled, false, "BigInt.prototype.toString will not be called");
  assert(compareArray(result, [3n, 20n, 100n]));
}, null, null, ["immutable"]);
