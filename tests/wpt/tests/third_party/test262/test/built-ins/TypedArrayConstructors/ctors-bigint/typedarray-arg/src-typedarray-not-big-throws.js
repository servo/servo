// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: >
  If typedArray constructor argument is not a Big(U)Int, throw
info: |
  22.2.4.3 TypedArray ( typedArray )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has a [[TypedArrayName]] internal slot.

  ...
  19. Else,
    ...
    c. If one of srcType and elementType contains the substring "Big" and the other
       does not, throw a TypeError exception.

includes: [testTypedArray.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var notBigTypedArray;

testWithTypedArrayConstructors(function(TA, makeCtorArg) {

  notBigTypedArray = new TA(makeCtorArg(16));

  testWithBigIntTypedArrayConstructors(function(BTA, makeCtorArg) {
    assert.throws(TypeError, function() {
      new BTA(notBigTypedArray);
    });
  });

});
