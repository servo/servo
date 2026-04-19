// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-sharedarraybuffer.prototype.growable
description: Return value according to [[ArrayBufferMaxByteLength]] internal slot
info: |
  get SharedArrayBuffer.prototype.growable

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is false, throw a TypeError exception.
  4. Return IsResizableArrayBuffer(O).

  IsResizableArrayBuffer ( arrayBuffer )

  1. Assert: Type(arrayBuffer) is Object and arrayBuffer has an
     [[ArrayBufferData]] internal slot.
  2. If buffer has an [[ArrayBufferMaxByteLength]] internal slot, return true.
  3. Return false.
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var sab1 = new SharedArrayBuffer(1);

assert.sameValue(sab1.growable, false);

var sab2 = new SharedArrayBuffer(1, {maxByteLength: 1});

assert.sameValue(sab2.growable, true);
