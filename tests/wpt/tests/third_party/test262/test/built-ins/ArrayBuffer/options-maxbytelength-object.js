// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer-constructor
description: |
  Invoked with an options object whose `maxByteLength` property cannot be
  coerced to a primitive value
info: |
  ArrayBuffer( length [ , options ] )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Let requestedMaxByteLength be ? GetArrayBufferMaxByteLengthOption(options).
  [...]

  1.1.5 GetArrayBufferMaxByteLengthOption ( options )

  1. If Type(options) is not Object, return empty.
  2. Let maxByteLength be ? Get(options, "maxByteLength").
  3. If maxByteLength is undefined, return empty.
  4. Return ? ToIndex(maxByteLength).
features: [resizable-arraybuffer]
---*/

var log = [];
var options = {
  maxByteLength: {
    toString: function() {
      log.push('toString');
      return {};
    },
    valueOf: function() {
      log.push('valueOf');
      return {};
    }
  }
};

assert.throws(TypeError, function() {
  new ArrayBuffer(0, options);
});

assert.sameValue(log.length, 2);
assert.sameValue(log[0], 'valueOf');
assert.sameValue(log[1], 'toString');
