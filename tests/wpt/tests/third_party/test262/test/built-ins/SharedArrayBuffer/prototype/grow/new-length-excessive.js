// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer.prototype.grow
description: >
  Throws a RangeError the newLength value is larger than the max byte length
info: |
  SharedArrayBuffer.prototype.grow ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  3. If IsSharedArrayBuffer(O) is false, throw a TypeError exception.
  4. Let newByteLength be ? ToIntegerOrInfinity(newLength).
  5. If newByteLength < 0 or newByteLength > O.[[ArrayBufferMaxByteLength]],
     throw a RangeError exception.
  [...]
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var ab = new SharedArrayBuffer(4, {maxByteLength: 4});

assert.throws(RangeError, function() {
  ab.grow(5);
});
