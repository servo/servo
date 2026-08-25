// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.dotall
description: >
    `dotAll` accessor invoked on a non-object value
info: |
    get RegExp.prototype.dotAll

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol, regexp-dotall]
---*/

var dotAll = Object.getOwnPropertyDescriptor(RegExp.prototype, "dotAll").get;

assert.throws(TypeError, function() {
  dotAll.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  dotAll.call(null);
}, "null");

assert.throws(TypeError, function() {
  dotAll.call(true);
}, "true");

assert.throws(TypeError, function() {
  dotAll.call("string");
}, "string");

assert.throws(TypeError, function() {
  dotAll.call(Symbol("s"));
}, "symbol");

assert.throws(TypeError, function() {
  dotAll.call(4);
}, "number");

assert.throws(TypeError, function() {
  dotAll.call(4n);
}, "bigint");
