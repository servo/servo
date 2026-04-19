// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  ToInteger(offset) operations
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
  7. If targetOffset < 0, throw a RangeError exception.
  ...
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], "");
  assert(compareArray(sample, [42n, 2n]), "the empty string");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], "0");
  assert(compareArray(sample, [42n, 2n]), "'0'");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], false);
  assert(compareArray(sample, [42n, 2n]), "false");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], 0.1);
  assert(compareArray(sample, [42n, 2n]), "0.1");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], 0.9);
  assert(compareArray(sample, [42n, 2n]), "0.9");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], -0.5);
  assert(compareArray(sample, [42n, 2n]), "-0.5");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], 1.1);
  assert(compareArray(sample, [1n, 42n]), "1.1");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], NaN);
  assert(compareArray(sample, [42n, 2n]), "NaN");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], null);
  assert(compareArray(sample, [42n, 2n]), "null");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], undefined);
  assert(compareArray(sample, [42n, 2n]), "undefined");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], {});
  assert(compareArray(sample, [42n, 2n]), "{}");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], []);
  assert(compareArray(sample, [42n, 2n]), "[]");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], [0]);
  assert(compareArray(sample, [42n, 2n]), "[0]");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], true);
  assert(compareArray(sample, [1n, 42n]), "true");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], "1");
  assert(compareArray(sample, [1n, 42n]), "'1'");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], [1]);
  assert(compareArray(sample, [1n, 42n]), "[1]");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], { valueOf: function() {return 1;} });
  assert(compareArray(sample, [1n, 42n]), "valueOf");

  sample = new TA(makeCtorArg([1n, 2n]));
  sample.set([42n], { toString: function() {return 1;} });
  assert(compareArray(sample, [1n, 42n]), "toString");
});
