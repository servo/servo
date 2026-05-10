// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reverse
description: Reverts values
info: |
  22.2.3.22 %TypedArray%.prototype.reverse ( )

  %TypedArray%.prototype.reverse is a distinct function that implements the same
  algorithm as Array.prototype.reverse as defined in 22.1.3.21 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.21 Array.prototype.reverse ( )

  ...
  6. Return O.
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

var buffer = new ArrayBuffer(64);

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA(buffer, 0, 4);
  var other = new TA(buffer, 0, 5);

  sample[0] = 42n;
  sample[1] = 43n;
  sample[2] = 2n;
  sample[3] = 1n;
  other[4] = 7n;

  sample.reverse();
  assert(
    compareArray(sample, [1n, 2n, 43n, 42n])
  );

  assert(
    compareArray(other, [1n, 2n, 43n, 42n, 7n])
  );

  sample[0] = 7n;
  sample[1] = 17n;
  sample[2] = 1n;
  sample[3] = 0n;
  other[4] = 42n;

  other.reverse();
  assert(
    compareArray(other, [42n, 0n, 1n, 17n, 7n])
  );

  assert(
    compareArray(sample, [42n, 0n, 1n, 17n])
  );
}, null, ["passthrough"]);
