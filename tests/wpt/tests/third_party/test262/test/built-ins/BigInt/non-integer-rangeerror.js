// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Non integer number values will throw a RangeError
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
  3. Let integer be ToInteger(number).
  4. If integer is not equal to number, return false.
  ...
features: [BigInt]
---*/

assert.throws(RangeError, function() {
  BigInt(0.00005);
});

assert.throws(RangeError, function() {
  BigInt(-0.00005);
});

assert.throws(RangeError, function() {
  BigInt(.1);
});

assert.throws(RangeError, function() {
  BigInt(-.1);
});

assert.throws(RangeError, function() {
  BigInt(1.1);
});

assert.throws(RangeError, function() {
  BigInt(-1.1);
});

assert.throws(RangeError, function() {
  BigInt(Number.MIN_VALUE);
});
