// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  [[Get]] of "length" uses [[ArrayLength]]
info: |
  22.2.3.10 %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  ...
  3. Let len be O.[[ArrayLength]].
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray, array-find-from-last]
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
    sample.findLast(function() { return true; }),
    42n
  );
}, null, ["passthrough"]);
