// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.from
description: >
  Throws a TypeError if a custom `this` returns a smaller instance
info: |
  %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  7. If usingIterator is not undefined, then
    a. Let values be ? IterableToList(source, usingIterator).
    b. Let len be the number of elements in values.
    c. Let targetObj be ? TypedArrayCreate(C, «len»).
  ...
  10. Let len be ? ToLength(? Get(arrayLike, "length")).
  11. Let targetObj be ? TypedArrayCreate(C, « len »).
  ...
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray]
---*/

var sourceItor = [1, 2];
var sourceObj = {
  length: 2
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ctor = function() {
    return new TA(makeCtorArg(1));
  };
  assert.throws(TypeError, function() {
    TA.from.call(ctor, sourceItor);
  }, "source is using iterator");

  assert.throws(TypeError, function() {
    TA.from.call(ctor, sourceObj);
  }, "source is not using iterator");
});
