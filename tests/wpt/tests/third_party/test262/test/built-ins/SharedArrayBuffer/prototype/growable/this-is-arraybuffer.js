// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-sharedarraybuffer.prototype.growable
description: Throws a TypeError exception when `this` is an ArrayBuffer
info: |
  get SharedArrayBuffer.prototype.growable

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is false, throw a TypeError exception.
  [...]
features: [ArrayBuffer, SharedArrayBuffer, resizable-arraybuffer]
---*/

var growable = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, "growable"
);

var getter = growable.get;
var ab = new ArrayBuffer(4);

assert.sameValue(typeof getter, "function");

assert.throws(TypeError, function() {
  getter.call(ab);
}, "`this` cannot be an ArrayBuffer");

Object.defineProperties(ab, { growable: growable });

assert.throws(TypeError, function() {
  ab.growable;
}, "`this` cannot be an ArrayBuffer");
