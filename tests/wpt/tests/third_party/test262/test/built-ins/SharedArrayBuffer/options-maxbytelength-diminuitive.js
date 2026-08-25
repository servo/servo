// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer-constructor
description: |
  Invoked with an options object whose `maxByteLength` property is less than
  the length.
info: |
  SharedArrayBuffer( length [ , options ] )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Let requestedMaxByteLength be ? GetArrayBufferMaxByteLengthOption(options).
  4. If requestedMaxByteLength is empty, then
     a. [...]
  5. If byteLength > requestedMaxByteLength, throw a RangeError exception.
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

assert.throws(RangeError, function() {
  new SharedArrayBuffer(1, {maxByteLength: 0});
});
