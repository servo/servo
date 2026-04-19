// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  ToInteger(offset) operations
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;
  var src = new TA(makeCtorArg([42]));

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, "");
  assert(compareArray(sample, [42, 2]), "the empty string");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, "0");
  assert(compareArray(sample, [42, 2]), "'0'");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, false);
  assert(compareArray(sample, [42, 2]), "false");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, 0.1);
  assert(compareArray(sample, [42, 2]), "0.1");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, 0.9);
  assert(compareArray(sample, [42, 2]), "0.9");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, -0.5);
  assert(compareArray(sample, [42, 2]), "-0.5");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, 1.1);
  assert(compareArray(sample, [1, 42]), "1.1");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, NaN);
  assert(compareArray(sample, [42, 2]), "NaN");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, null);
  assert(compareArray(sample, [42, 2]), "null");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, undefined);
  assert(compareArray(sample, [42, 2]), "undefined");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, {});
  assert(compareArray(sample, [42, 2]), "{}");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, []);
  assert(compareArray(sample, [42, 2]), "[]");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, [0]);
  assert(compareArray(sample, [42, 2]), "[0]");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, true);
  assert(compareArray(sample, [1, 42]), "true");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, "1");
  assert(compareArray(sample, [1, 42]), "'1'");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, [1]);
  assert(compareArray(sample, [1, 42]), "[1]");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, { valueOf: function() {return 1;} });
  assert(compareArray(sample, [1, 42]), "valueOf");

  sample = new TA(makeCtorArg([1, 2]));
  sample.set(src, { toString: function() {return 1;} });
  assert(compareArray(sample, [1, 42]), "toString");
});
