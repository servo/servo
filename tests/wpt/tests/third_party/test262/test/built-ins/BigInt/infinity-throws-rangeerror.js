// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: BigInt throws a RangeError if value is Infinity
esid: sec-bigint-constructor
info: |
  BigInt ( value )

  ...
  2. Let prim be ? ToPrimitive(value, hint Number).
  3. If Type(prim) is Number, return ? NumberToBigInt(prim).
  ...

  NumberToBigInt ( number )

  ...
  2. If IsSafeInteger(number) is false, throw a RangeError exception.
  ...

  IsSafeInteger ( number )

  ...
  2. If number is NaN, +∞, or -∞, return false.
features: [BigInt]
---*/

assert.throws(RangeError, function() {
  BigInt(Infinity);
});

var calls = 0;
var obj = {
  valueOf: function() {
    calls++;
    return Infinity;
  }
}
assert.throws(RangeError, function() {
  BigInt(obj);
});
assert.sameValue(calls, 1, "it fails after fetching the primitive value");
