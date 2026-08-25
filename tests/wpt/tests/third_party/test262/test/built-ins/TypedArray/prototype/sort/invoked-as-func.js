// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: Throws a TypeError exception when invoked as a function
info: |
  22.2.3.25 %TypedArray%.prototype.sort ( comparefn )

  ...
  This function is not generic. The this value must be an object with a
  [[TypedArrayName]] internal slot.
  ...

  1. Let obj be the this value as the argument.
  2. Let buffer be ValidateTypedArray(obj).
  3. ReturnIfAbrupt(buffer).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  2. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var sort = TypedArray.prototype.sort;

assert.sameValue(typeof sort, 'function');

assert.throws(TypeError, function() {
  sort();
});
