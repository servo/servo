// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.from
description: >
  Custom constructor can return any TypedArray instance with higher or same
  length
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
features: [BigInt, Symbol.iterator, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sourceItor = [1n, 2n];
  var sourceObj = {
    0: 0n,
    1: 0n,
    length: 2
  };

  var result;
  var custom = new TA(makeCtorArg(2));
  var ctor = function() {
    return custom;
  };

  result = TypedArray.from.call(ctor, sourceItor);
  assert.sameValue(result, custom, "using iterator, same length");

  result = TypedArray.from.call(ctor, sourceObj);
  assert.sameValue(result, custom, "not using iterator, same length");

  custom = new TA(makeCtorArg(3));

  result = TypedArray.from.call(ctor, sourceItor);
  assert.sameValue(result, custom, "using iterator, higher length");

  result = TypedArray.from.call(ctor, sourceObj);
  assert.sameValue(result, custom, "not using iterator, higher length");
});
