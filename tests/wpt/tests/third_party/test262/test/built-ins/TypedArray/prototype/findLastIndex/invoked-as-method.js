// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: Requires a [[TypedArrayName]] internal slot.
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  Perform ? ValidateTypedArray(O).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  2. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

var TypedArrayPrototype = TypedArray.prototype;

assert.sameValue(typeof TypedArrayPrototype.findLastIndex, 'function');

assert.throws(TypeError, function() {
  TypedArrayPrototype.findLastIndex();
});
