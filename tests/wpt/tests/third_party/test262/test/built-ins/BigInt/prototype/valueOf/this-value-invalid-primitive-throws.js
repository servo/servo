// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.valueof
description: >
  Throws a TypeError if this is not a BigInt neither an Object.
info: |
  BigInt.prototype.valueOf ( )

  1. Return ? thisBigIntValue(this value).

  The abstract operation thisBigIntValue(value) performs the following steps:

  1. If Type(value) is BigInt, return value.
  2. If Type(value) is Object and value has a [[BigIntData]] internal slot, then
    ...
  3. Throw a TypeError exception.
features: [BigInt, Symbol]
---*/

var valueOf = BigInt.prototype.valueOf;

assert.sameValue(typeof valueOf, 'function');

assert.throws(TypeError, function() {
  valueOf.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  valueOf.call(null);
}, "null");

assert.throws(TypeError, function() {
  valueOf.call(false);
}, "false");

assert.throws(TypeError, function() {
  valueOf.call(true);
}, "true");

assert.throws(TypeError, function() {
  valueOf.call("");
}, "the empty string");

assert.throws(TypeError, function() {
  valueOf.call("1n");
}, "string");

assert.throws(TypeError, function() {
  valueOf.call(0);
}, "number (0)");

assert.throws(TypeError, function() {
  valueOf.call(1);
}, "number (1)");

assert.throws(TypeError, function() {
  valueOf.call(NaN);
}, "NaN");

var s = Symbol();
assert.throws(TypeError, function() {
  valueOf.call(s);
}, "symbol");
