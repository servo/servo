// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt.asIntN type coercion for bigint parameter
esid: sec-bigint.asintn
info: |
  BigInt.asIntN ( bits, bigint )

  2. Let bigint ? ToBigInt(bigint).
features: [BigInt, computed-property-names, Symbol, Symbol.toPrimitive]
---*/

assert.sameValue(BigInt.asIntN(2, Object(0n)), 0n, "ToPrimitive: unbox object with internal slot");
assert.sameValue(BigInt.asIntN(2, {
  [Symbol.toPrimitive]: function() {
    return 0n;
  }
}), 0n, "ToPrimitive: @@toPrimitive");
assert.sameValue(BigInt.asIntN(2, {
  valueOf: function() {
    return 0n;
  }
}), 0n, "ToPrimitive: valueOf");
assert.sameValue(BigInt.asIntN(2, {
  toString: function() {
    return 0n;
  }
}), 0n, "ToPrimitive: toString");
assert.sameValue(BigInt.asIntN(2, Object(true)), 1n,
  "ToBigInt: unbox object with internal slot => true => 1n");
assert.sameValue(BigInt.asIntN(2, {
  [Symbol.toPrimitive]: function() {
    return true;
  }
}), 1n, "ToBigInt: @@toPrimitive => true => 1n");
assert.sameValue(BigInt.asIntN(2, {
  valueOf: function() {
    return true;
  }
}), 1n, "ToBigInt: valueOf => true => 1n");
assert.sameValue(BigInt.asIntN(2, {
  toString: function() {
    return true;
  }
}), 1n, "ToBigInt: toString => true => 1n");
assert.sameValue(BigInt.asIntN(2, Object("1")), 1n,
  "ToBigInt: unbox object with internal slot => parse BigInt");
assert.sameValue(BigInt.asIntN(2, {
  [Symbol.toPrimitive]: function() {
    return "1";
  }
}), 1n, "ToBigInt: @@toPrimitive => parse BigInt");
assert.sameValue(BigInt.asIntN(2, {
  valueOf: function() {
    return "1";
  }
}), 1n, "ToBigInt: valueOf => parse BigInt");
assert.sameValue(BigInt.asIntN(2, {
  toString: function() {
    return "1";
  }
}), 1n, "ToBigInt: toString => parse BigInt");
