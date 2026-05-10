// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  Verify predicate this on non-strict mode
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  4. If thisArg was supplied, let T be thisArg; else let T be undefined.
  5. Let k be 0.
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
  ...
flags: [noStrict]
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var T = this;

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));
  var result;

  sample.findIndex(function() {
    result = this;
  });

  assert.sameValue(result, T, "without thisArg, predicate this is the global");

  result = null;  
  sample.findIndex(function() {
    result = this;
  }, undefined);

  assert.sameValue(result, T, "predicate this is the global when thisArg is undefined");

  var o = {};
  result = null;
  sample.findIndex(function() {
    result = this;
  }, o);

  assert.sameValue(result, o, "thisArg becomes the predicate this");
});
