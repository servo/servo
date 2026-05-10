// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.entries
description: >
  The prototype of the returned iterator is ArrayIteratorPrototype
info: |
  22.2.3.6 %TypedArray%.prototype.entries ( )

  ...
  3. Return CreateArrayIterator(O, "key+value").
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray]
---*/

var ArrayIteratorProto = Object.getPrototypeOf([][Symbol.iterator]());

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([0, 42, 64]));
  var iter = sample.entries();

  assert.sameValue(Object.getPrototypeOf(iter), ArrayIteratorProto);
});
