// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  Does not copy non-integer properties to returned value
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

  ...
  8. Repeat, while k < len
    a. Let Pk be ! ToString(k).
    b. Let kValue be ? Get(O, Pk).
    c. Let mappedValue be ? Call(callbackfn, T, « kValue, k, O »).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([7n, 8n]));
  var bar = Symbol("1");

  sample.foo = 42;
  sample[bar] = 1;

  var result = sample.map(function() {
    return 0n;
  });

  assert.sameValue(result.length, 2, "result.length");
  assert.sameValue(
    Object.getOwnPropertyDescriptor(result, "foo"),
    undefined,
    "foo"
  );
  assert.sameValue(
    Object.getOwnPropertyDescriptor(result, bar),
    undefined,
    "bar"
  );
});
