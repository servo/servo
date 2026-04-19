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

  assert.sameValue(a.at(false), 0, 'a.at(false) must return 0');
  assert.sameValue(a.at(null), 0, 'a.at(null) must return 0');
  assert.sameValue(a.at(undefined), 0, 'a.at(undefined) must return 0');
  assert.sameValue(a.at(""), 0, 'a.at("") must return 0');
  assert.sameValue(a.at(function() {}), 0, 'a.at(function() {}) must return 0');
  assert.sameValue(a.at([]), 0, 'a.at([]) must return 0');

  assert.sameValue(a.at(true), 1, 'a.at(true) must return 1');
  assert.sameValue(a.at("1"), 1, 'a.at("1") must return 1');
});
