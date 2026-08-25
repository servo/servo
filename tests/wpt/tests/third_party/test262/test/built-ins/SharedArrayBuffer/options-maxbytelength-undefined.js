// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer-constructor
description: Invoked with an options object whose `maxByteLength` property is undefined
info: |
  SharedArrayBuffer( length [ , options ] )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Let requestedMaxByteLength be ? GetArrayBufferMaxByteLengthOption(options).
  4. If requestedMaxByteLength is empty, then
     a. Return ? AllocateSharedArrayBuffer(NewTarget, byteLength).

  1.1.5 GetArrayBufferMaxByteLengthOption ( options )

  1. If Type(options) is not Object, return empty.
  2. Let maxByteLength be ? Get(options, "maxByteLength").
  3. If maxByteLength is undefined, return empty.
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

assert.sameValue(new SharedArrayBuffer(0, {}).growable, false);
assert.sameValue(new SharedArrayBuffer(0, {maxByteLength: undefined}).growable, false);
