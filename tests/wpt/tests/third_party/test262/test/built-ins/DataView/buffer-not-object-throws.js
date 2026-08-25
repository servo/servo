// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Throws a TypeError if buffer is not Object
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. If Type(buffer) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var obj = {
  valueOf: function() {
    throw new Test262Error("buffer should be verified before byteOffset");
  }
};

assert.throws(TypeError, function() {
  new DataView(0, obj);
}, "0");

assert.throws(TypeError, function() {
  new DataView(1, obj);
}, "1");

assert.throws(TypeError, function() {
  new DataView("", obj);
}, "the empty string");

assert.throws(TypeError, function() {
  new DataView("buffer", obj);
}, "string");

assert.throws(TypeError, function() {
  new DataView(false, obj);
}, "false");

assert.throws(TypeError, function() {
  new DataView(true, obj);
}, "true");

assert.throws(TypeError, function() {
  new DataView(NaN, obj);
}, "NaN");

assert.throws(TypeError, function() {
  new DataView(Symbol("1"), obj);
}, "symbol");
