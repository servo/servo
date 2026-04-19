// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: Instantiate a new TypedArray with an existing TypedArray
info: |
  22.2.4.3 TypedArray ( typedArray )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has a [[TypedArrayName]] internal slot.

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var sample1 = new BigInt64Array(7);
var sample2 = new BigUint64Array(7);

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = TA === BigInt64Array ? sample2 : sample1;
  var typedArray = new TA(sample);

  assert.sameValue(typedArray.length, 7);
  assert.notSameValue(typedArray, sample);
  assert.sameValue(typedArray.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(typedArray), TA.prototype);
}, null, ["passthrough"]);
