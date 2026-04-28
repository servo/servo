// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-sharedarraybuffer.prototype.growable
description: >
  Throws a TypeError exception when `this` does not have a [[ArrayBufferData]]
  internal slot
info: |
  get SharedArrayBuffer.prototype.growable

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]
features: [DataView, SharedArrayBuffer, TypedArray, resizable-arraybuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, "growable"
).get;

assert.sameValue(typeof getter, "function");

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

var dv = new DataView(new SharedArrayBuffer(8), 0);
assert.throws(TypeError, function() {
  getter.call(dv);
});
