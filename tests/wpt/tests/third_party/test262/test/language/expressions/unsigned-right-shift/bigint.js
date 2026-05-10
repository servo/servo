// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Unsigned right shift always throws for BigInt values
esid: sec-numeric-types-bigint-unsignedRightShift
info: |
  BigInt::unsignedRightShift (x, y)

  The abstract operation BigInt::unsignedRightShift with two arguments x and y of type BigInt:

  1. Throw a TypeError exception.

features: [BigInt]
---*/

assert.throws(TypeError, function() { 0n >>> 0n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 5n >>> 1n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 5n >>> 2n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 5n >>> 3n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 5n >>> -1n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 5n >>> -2n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 5n >>> -3n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 0n >>> 128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 0n >>> -128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 582n >>> 0n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 582n >>> 127n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 582n >>> 128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 582n >>> 129n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 582n >>> -128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> 64n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> 32n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> 16n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> 0n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> -16n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> -32n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> -64n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> -127n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> -128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { 405972677036361916727469983882855107238581880n >>> -129n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -5n >>> 1n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -5n >>> 2n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -5n >>> 3n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -5n >>> -1n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -5n >>> -2n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -5n >>> -3n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -1n >>> 128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -1n >>> 0n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -1n >>> -128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -582n >>> 0n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -582n >>> 127n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -582n >>> 128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -582n >>> 129n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -582n >>> -128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> 64n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> 32n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> 16n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> 0n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> -16n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> -32n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> -64n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> -127n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> -128n; }, "bigint >>> bigint throws a TypeError");
assert.throws(TypeError, function() { -405972677036361916727469983882855107238581880n >>> -129n; }, "bigint >>> bigint throws a TypeError");
