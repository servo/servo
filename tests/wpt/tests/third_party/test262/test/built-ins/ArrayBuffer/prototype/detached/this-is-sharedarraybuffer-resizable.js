// Copyright (C) 2023 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.detached
description: Throws a TypeError exception when `this` is a resizable SharedArrayBuffer
info: |
  get ArrayBuffer.prototype.detached

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  [...]
features: [SharedArrayBuffer, ArrayBuffer, arraybuffer-transfer]
---*/

var detached = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, "detached"
);

var getter = detached.get;
var sab = new SharedArrayBuffer(4);

assert.sameValue(typeof getter, "function");

assert.throws(TypeError, function() {
  getter.call(sab);
}, "`this` cannot be a SharedArrayBuffer");
