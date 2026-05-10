// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.bytelength
description: >
  Throws a TypeError exception when `this` does not have a [[ArrayBufferData]]
  internal slot
info: |
  24.1.4.1 get ArrayBuffer.prototype.byteLength

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have an [[ArrayBufferData]] internal slot, throw a TypeError
  exception.
  ...
features: [DataView, Int8Array]
---*/

var getter = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, "byteLength"
).get;

assert.throws(TypeError, function() {
  getter.call({});
});

assert.throws(TypeError, function() {
  getter.call([]);
});

var ta = new Int8Array(8);
assert.throws(TypeError, function() {
  getter.call(ta);
});

var dv = new DataView(new ArrayBuffer(8), 0);
assert.throws(TypeError, function() {
  getter.call(dv);
});
