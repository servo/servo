// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: >
  If `searchElement` is not supplied, -1 is returned.
info: |
  %TypedArray%.prototype.indexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements
  the same algorithm as Array.prototype.indexOf as defined in 22.1.3.14
  except that the this value's [[ArrayLength]] internal slot is accessed
  in place of performing a [[Get]] of "length".

  Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  [...]
  10. Return -1.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta1 = new TA();
  assert.sameValue(ta1.indexOf(), -1);

  var ta2 = new TA(makeCtorArg([0, 1, 2]));
  assert.sameValue(ta2.indexOf(), -1);
});
