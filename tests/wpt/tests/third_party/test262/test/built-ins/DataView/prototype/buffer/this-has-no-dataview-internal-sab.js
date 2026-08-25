// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.buffer
description: >
  Throws a TypeError exception when `this` does not have a [[DataView]] internal
  slot
info: |
  24.2.4.1 get DataView.prototype.buffer

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have a [[DataView]] internal slot, throw a TypeError
  exception.
  ...
features: [SharedArrayBuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  DataView.prototype, "buffer"
).get;

assert.throws(TypeError, function() {
  getter.call({});
}, "{}");

assert.throws(TypeError, function() {
  getter.call([]);
}, "[]");

var ab = new SharedArrayBuffer(8);
assert.throws(TypeError, function() {
  getter.call(ab);
}, "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  getter.call(ta);
}, "TypedArray");
