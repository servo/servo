// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )
features: [SharedArrayBuffer, Symbol]
---*/

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call(undefined);
}, "`this` value is undefined");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call(null);
}, "`this` value is null");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call(true);
}, "`this` value is Boolean");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call("");
}, "`this` value is String");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call(Symbol());
}, "`this` value is Symbol");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call(1);
}, "`this` value is Number");
