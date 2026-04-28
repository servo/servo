// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
  non callable argument should throw TypeError Exception
info: |
  Array.prototype.flatMap ( mapperFunction [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let sourceLen be ? ToLength(? Get(O, "length")).
  3. If IsCallable(mapperFunction) is false, throw a TypeError exception.
  ...
features: [Array.prototype.flatMap, Symbol]
---*/

assert.sameValue(typeof Array.prototype.flatMap, "function");

assert.throws(TypeError, function() {
  [].flatMap({});
}, 'non callable argument, object');

assert.throws(TypeError, function() {
  [].flatMap(0);
}, 'non callable argument, number');

assert.throws(TypeError, function() {
  [].flatMap();
}, 'non callable argument, implict undefined');

assert.throws(TypeError, function() {
  [].flatMap(undefined);
}, 'non callable argument, undefined');

assert.throws(TypeError, function() {
  [].flatMap(null);
}, 'non callable argument, null');

assert.throws(TypeError, function() {
  [].flatMap(false);
}, 'non callable argument, boolean');

assert.throws(TypeError, function() {
  [].flatMap('');
}, 'non callable argument, string');

var s = Symbol();
assert.throws(TypeError, function() {
  [].flatMap(s);
}, 'non callable argument, symbol');
