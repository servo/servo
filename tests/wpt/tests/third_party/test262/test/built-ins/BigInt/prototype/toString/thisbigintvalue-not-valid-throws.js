// Copyright 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tostring
description: Throws a TypeError if the this value is not a BigInt
info: |
  BigInt.prototype.toString ( [ radix ] )

  1. Let x be ? thisBigIntValue(this value).
  ...

  The abstract operation thisBigIntValue(value) performs the following steps:

  1. If Type(value) is BigInt, return value.
  2. If Type(value) is Object and value has a [[BigIntData]] internal slot, then
    ...
  3. Throw a TypeError exception.
features: [BigInt, Symbol, Symbol.toPrimitive]
---*/

var toString = BigInt.prototype.toString;

assert.sameValue(typeof toString, 'function');

assert.throws(TypeError, function() {
  toString.call({
    x: 1n
  });
}, '{x: 1n}');

assert.throws(TypeError, function() {
  toString.call([1n]);
}, '[1n]');

var obj = {
  valueOf: function() {
    throw new Test262Error('no [[BigIntData]]')
  },
  toString: function() {
    throw new Test262Error('no [[BigIntData]]')
  },
  [Symbol.toPrimitive]: function() {
    throw new Test262Error('no [[BigIntData]]')
  }
};
assert.throws(TypeError, function() {
  toString.call(obj);
}, '{valueOf, toString, toPrimitive}');

assert.throws(TypeError, function() {
  toString.call(0);
}, '0');

assert.throws(TypeError, function() {
  toString.call(1);
}, '1');

assert.throws(TypeError, function() {
  toString.call(NaN);
}, 'NaN');

assert.throws(TypeError, function() {
  toString.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  toString.call(null);
}, 'null');

assert.throws(TypeError, function() {
  toString.call(true);
}, 'true');

assert.throws(TypeError, function() {
  toString.call(false);
}, 'false');
