// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.at
description: >
  Returns undefined if the specified index less than or greater than the available index range.
info: |
  %TypedArray%.prototype.at( index )

  If k < 0 or k ≥ len, then return undefined.

includes: [testTypedArray.js]
features: [TypedArray,TypedArray.prototype.at]
---*/
assert.sameValue(
  typeof TypedArray.prototype.at,
  'function',
  'The value of `typeof TypedArray.prototype.at` is "function"'
);

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  assert.sameValue(typeof TA.prototype.at, 'function', 'The value of `typeof TA.prototype.at` is "function"');
  let a = new TA(makeCtorArg([]));

  assert.sameValue(a.at(-2), undefined, 'a.at(-2) must return undefined'); // wrap around the end
  assert.sameValue(a.at(0), undefined, 'a.at(0) must return undefined');
  assert.sameValue(a.at(1), undefined, 'a.at(1) must return undefined');
});
