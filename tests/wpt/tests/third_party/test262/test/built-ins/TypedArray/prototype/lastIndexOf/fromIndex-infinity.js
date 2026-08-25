// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: handle Infinity values for fromIndex
info: |
  22.2.3.17 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.lastIndexOf is a distinct function that implements the
  same algorithm as Array.prototype.lastIndexOf as defined in 22.1.3.15 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.15 Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  5. If n ≥ 0, then
    a. If n is -0, let k be +0; else let k be min(n, len - 1).
  6. Else n < 0,
    a. Let k be len + n.
  7. Repeat, while k ≥ 0
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43, 43, 41]));

  assert.sameValue(sample.lastIndexOf(43, Infinity), 2, "lastIndexOf(43, Infinity)");
  assert.sameValue(sample.lastIndexOf(43, -Infinity), -1, "lastIndexOf(43, -Infinity)");
});
