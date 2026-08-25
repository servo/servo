// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  [[Get]] of "length" uses [[ArrayLength]]
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

Object.defineProperty(TypedArray.prototype, "length", {
  get: function() {
    throw new Test262Error();
  }
});

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  Object.defineProperty(TA.prototype, "length", {
    get: function() {
      throw new Test262Error();
    }
  });

  var sample = new TA(makeCtorArg([42n]));

  Object.defineProperty(sample, "length", {
    get: function() {
      throw new Test262Error();
    },
    configurable: true
  });

  assert.sameValue(
    sample.findIndex(function() { return true; }),
    0
  );
}, null, ["passthrough"]);
