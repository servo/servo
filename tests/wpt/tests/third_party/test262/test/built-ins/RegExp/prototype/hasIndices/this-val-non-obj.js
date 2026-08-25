// Copyright (C) 2021 Ron Buckton and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.hasindices
description: >
    `hasIndices` accessor invoked on a non-object value
info: |
    get RegExp.prototype.hasIndices

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol, regexp-match-indices]
---*/

var hasIndices = Object.getOwnPropertyDescriptor(RegExp.prototype, "hasIndices").get;

assert.throws(TypeError, function() {
  hasIndices.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  hasIndices.call(null);
}, "null");

assert.throws(TypeError, function() {
  hasIndices.call(true);
}, "true");

assert.throws(TypeError, function() {
  hasIndices.call("string");
}, "string");

assert.throws(TypeError, function() {
  hasIndices.call(Symbol("s"));
}, "symbol");

assert.throws(TypeError, function() {
  hasIndices.call(4);
}, "number");

assert.throws(TypeError, function() {
  hasIndices.call(4n);
}, "bigint");
