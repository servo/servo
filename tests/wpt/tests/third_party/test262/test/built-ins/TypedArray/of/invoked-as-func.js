// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 22.2.2.2
description: >
  "of" cannot be invoked as a function
info: |
  22.2.2.2 %TypedArray%.of ( ...items )

  ...
  3. Let C be the this value.
  4. If IsConstructor(C) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var of = TypedArray.of;

assert.throws(TypeError, function() {
  of();
});
