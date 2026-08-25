// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: >
  If `searchElement` is not supplied, -1 is returned.
info: |
  %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.lastIndexOf is a distinct function that implements
  the same algorithm as Array.prototype.lastIndexOf as defined in 22.1.3.17
  except that the this value's [[ArrayLength]] internal slot is accessed
  in place of performing a [[Get]] of "length".

  Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  [...]
  8. Return -1.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta1 = new TA();
  assert.sameValue(ta1.lastIndexOf(), -1);

  var ta2 = new TA(makeCtorArg([0, 1, 2]));
  assert.sameValue(ta2.lastIndexOf(), -1);
});
