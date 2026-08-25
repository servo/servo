// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Return does not share buffer
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  10. Let A be ? TypedArraySpeciesCreate(O, « captured »).
  ...
  13. Return A.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n]));
  var result;

  result = sample.filter(function() { return true; });
  assert.notSameValue(result.buffer, sample.buffer);

  result = sample.filter(function() { return false; });
  assert.notSameValue(result.buffer, sample.buffer);
});
