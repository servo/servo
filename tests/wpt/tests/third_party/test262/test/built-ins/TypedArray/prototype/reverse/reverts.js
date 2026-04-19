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
features: [TypedArray]
---*/

var buffer = new ArrayBuffer(64);

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA(buffer, 0, 4);
  var other = new TA(buffer, 0, 5);

  sample[0] = 42;
  sample[1] = 43;
  sample[2] = 2;
  sample[3] = 1;
  other[4] = 7;

  sample.reverse();
  assert(
    compareArray(sample, [1, 2, 43, 42])
  );

  assert(
    compareArray(other, [1, 2, 43, 42, 7])
  );

  sample[0] = 7;
  sample[1] = 17;
  sample[2] = 1;
  sample[3] = 0;
  other[4] = 42;

  other.reverse();
  assert(
    compareArray(other, [42, 0, 1, 17, 7])
  );

  assert(
    compareArray(sample, [42, 0, 1, 17])
  );
}, null, ["passthrough"]);
