// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Throws a TypeError exception if this is not a constructor
info: |
  22.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  1. Let C be the this value.
  2. If IsConstructor(C) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var from = TypedArray.from;
var m = { m() {} }.m;

assert.throws(TypeError, function() {
  from.call(m, []);
});
