// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer-constructor
description: Invoked with an options object whose `maxByteLength` property throws
info: |
  SharedArrayBuffer( length [ , options ] )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Let requestedMaxByteLength be ? GetArrayBufferMaxByteLengthOption(options).
  [...]

  1.1.5 GetArrayBufferMaxByteLengthOption ( options )

  1. If Type(options) is not Object, return empty.
  2. Let maxByteLength be ? Get(options, "maxByteLength").
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var options = {
  get maxByteLength() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  new SharedArrayBuffer(0, options);
});
