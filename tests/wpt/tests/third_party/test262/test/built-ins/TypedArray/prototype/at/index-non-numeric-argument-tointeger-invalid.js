// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.at
description: >
  Property type and descriptor.
info: |
  %TypedArray%.prototype.at( index )

  Let relativeIndex be ? ToInteger(index).

includes: [testTypedArray.js]
features: [TypedArray, TypedArray.prototype.at]
---*/
assert.sameValue(
  typeof TypedArray.prototype.at,
  'function',
  'The value of `typeof TypedArray.prototype.at` is "function"'
);

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  assert.sameValue(typeof TA.prototype.at, 'function', 'The value of `typeof TA.prototype.at` is "function"');
  let a = new TA(makeCtorArg([0,1,2,3]));

  assert.throws(TypeError, () => {
    a.at(Symbol());
  });
});
