// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt.asIntN type coercion for bits parameter
esid: sec-bigint.asintn
info: |
  BigInt.asIntN ( bits, bigint )

  1. Let bits be ? ToIndex(bits).
features: [BigInt, computed-property-names, Symbol, Symbol.toPrimitive]
---*/
assert.sameValue(typeof BigInt, 'function');
assert.sameValue(typeof BigInt.asIntN, 'function');

assert.throws(RangeError, function() {
  BigInt.asIntN(-1, 0n);
}, "ToIndex: throw when integerIndex < 0");
assert.throws(RangeError, function() {
  BigInt.asIntN(-2.5, 0n);
}, "ToIndex: throw when integerIndex < 0");
assert.throws(RangeError, function() {
  BigInt.asIntN("-2.5", 0n);
}, "ToIndex: parse Number => throw when integerIndex < 0");
assert.throws(RangeError, function() {
  BigInt.asIntN(-Infinity, 0n);
}, "ToIndex: throw when integerIndex < 0");
assert.throws(RangeError, function() {
  BigInt.asIntN(9007199254740992, 0n);
}, "ToIndex: throw when integerIndex > 2**53-1");
assert.throws(RangeError, function() {
  BigInt.asIntN(Infinity, 0n);
}, "ToIndex: throw when integerIndex > 2**53-1");
assert.throws(TypeError, function() {
  BigInt.asIntN(0n, 0n);
}, "ToIndex: BigInt => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(Object(0n), 0n);
}, "ToIndex: unbox object with internal slot => BigInt => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: function() {
      return 0n;
    }
  }, 0n);
}, "ToIndex: @@toPrimitive => BigInt => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: function() {
      return 0n;
    }
  }, 0n);
}, "ToIndex: valueOf => BigInt => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    toString: function() {
      return 0n;
    }
  }, 0n);
}, "ToIndex: toString => BigInt => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(Symbol("1"), 0n);
}, "ToIndex: Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(Object(Symbol("1")), 0n);
}, "ToIndex: unbox object with internal slot => Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: function() {
      return Symbol("1");
    }
  }, 0n);
}, "ToIndex: @@toPrimitive => Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: function() {
      return Symbol("1");
    }
  }, 0n);
}, "ToIndex: valueOf => Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    toString: function() {
      return Symbol("1");
    }
  }, 0n);
}, "ToIndex: toString => Symbol => TypeError");
