// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-regexp.prototype.multiline
description: A TypeError is thrown when the "this" value is not an Object
info: |
  1. Let R be the this value.
  2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, 'multiline').get;
var symbol = Symbol();

assert.throws(TypeError, function() {
  get.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  get.call(null);
}, 'null');

assert.throws(TypeError, function() {
  get.call(3);
}, 'number');

assert.throws(TypeError, function() {
  get.call('string');
}, 'string');

assert.throws(TypeError, function() {
  get.call(true);
}, 'boolean');

assert.throws(TypeError, function() {
  get.call(symbol);
}, 'symbol');

assert.throws(TypeError, function() {
  get.call(4n);
}, 'bigint');
