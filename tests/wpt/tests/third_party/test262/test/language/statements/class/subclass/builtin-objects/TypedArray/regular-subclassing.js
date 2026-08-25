// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 22.2.4
description: Subclassing TypedArrays
info: |
  22.2.4 The TypedArray Constructors

  ...

  The TypedArray constructors are designed to be subclassable. They may be used
  as the value of an extends clause of a class definition. Subclass constructors
  that intend to inherit the specified TypedArray behaviour must include a super
  call to the TypedArray constructor to create and initialize the subclass
  instance with the internal state necessary to support the
  %TypedArray%.prototype built-in methods.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(Constructor) {
  class Typed extends Constructor {}

  var arr = new Typed(2);

  assert.sameValue(arr.length, 2);
});
