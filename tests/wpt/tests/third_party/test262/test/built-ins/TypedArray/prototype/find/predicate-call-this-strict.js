// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: >
  Verify predicate this on strict mode
info: |
  22.2.3.10 %TypedArray%.prototype.find (predicate [ , thisArg ] )

  %TypedArray%.prototype.find is a distinct function that implements the same
  algorithm as Array.prototype.find as defined in 22.1.3.8 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse.

  ...

  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  4. If thisArg was supplied, let T be thisArg; else let T be undefined.
  ...
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
  ...
flags: [onlyStrict]
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));
  var result;

  sample.find(function() {
    result = this;
  });

  assert.sameValue(
    result,
    undefined,
    "without thisArg, predicate this is undefined"
  );

  var o = {};
  sample.find(function() {
    result = this;
  }, o);

  assert.sameValue(result, o, "thisArg becomes the predicate this");
});
