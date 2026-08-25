// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: Throws a TypeError if provided length cannot be coerced to a number
info: |
  ArrayBuffer.prototype.resize ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
  5. Let newByteLength be ? ToIntegerOrInfinity(newLength).
  [...]
features: [resizable-arraybuffer]
---*/

var log = [];
var newLength = {
  toString: function() {
    log.push('toString');
    return {};
  },
  valueOf: function() {
    log.push('valueOf');
    return {};
  }
};
var ab = new ArrayBuffer(0, {maxByteLength: 4});

assert.throws(TypeError, function() {
  ab.resize(newLength);
});

assert.sameValue(log.length, 2);
assert.sameValue(log[0], 'valueOf');
assert.sameValue(log[1], 'toString');
