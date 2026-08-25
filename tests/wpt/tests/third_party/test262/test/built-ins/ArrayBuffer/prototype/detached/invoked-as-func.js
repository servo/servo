// Copyright (C) 2023 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.detached
description: Returns true if the buffer is detached, else false
info: |
  get ArrayBuffer.prototype.detached

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]

features: [ArrayBuffer, arraybuffer-transfer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, 'detached'
).get;

assert.sameValue(typeof getter, 'function');

assert.throws(TypeError, function() {
  getter();
});
