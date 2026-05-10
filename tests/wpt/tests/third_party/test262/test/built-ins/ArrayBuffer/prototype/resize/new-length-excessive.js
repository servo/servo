// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: >
  Throws a RangeError the newLength value is larger than the max byte length
info: |
  ArrayBuffer.prototype.resize ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
  5. Let newByteLength be ? ToIntegerOrInfinity(newLength).
  6. If newByteLength < 0 or newByteLength > O.[[ArrayBufferMaxByteLength]],
     throw a RangeError exception.
  [...]
features: [resizable-arraybuffer]
---*/

var ab = new ArrayBuffer(4, {maxByteLength: 4});

assert.throws(RangeError, function() {
  ab.resize(5);
});
