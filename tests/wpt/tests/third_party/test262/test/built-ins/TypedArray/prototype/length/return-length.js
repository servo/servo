// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.length
description: >
  Return value from the [[ArrayLength]] internal slot
info: |
  22.2.3.18 get %TypedArray%.prototype.length

  ...
  6. Let length be the value of O's [[ArrayLength]] internal slot.
  7. Return length.

  ---

  The current tests on `prop-desc.js` and `length.js` already assert `length` is
  not a dynamic property as in regular arrays.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta1 = new TA();
  assert.sameValue(ta1.length, 0);

  var ta2 = new TA(makeCtorArg(42));
  assert.sameValue(ta2.length, 42);
});
