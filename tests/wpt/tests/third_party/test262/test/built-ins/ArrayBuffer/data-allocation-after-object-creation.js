// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  The new ArrayBuffer instance is created prior to allocating the Data Block.
info: |
  ArrayBuffer( length )

  ...
  6. Return AllocateArrayBuffer(NewTarget, byteLength).

  AllocateArrayBuffer( constructor, byteLength )
    1. Let obj be OrdinaryCreateFromConstructor(constructor, "%ArrayBufferPrototype%",
       «[[ArrayBufferData]], [[ArrayBufferByteLength]]» ).
    2. ReturnIfAbrupt(obj).
    ...
    4. Let block be CreateByteDataBlock(byteLength).
    5. ReturnIfAbrupt(block).
    ...
features: [Reflect.construct]
---*/

function DummyError() {}

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get: function() {
    throw new DummyError();
  }
});

assert.throws(DummyError, function() {
  // Allocating 7 PiB should fail with a RangeError.
  // Math.pow(1024, 5) = 1125899906842624
  Reflect.construct(ArrayBuffer, [7 * 1125899906842624], newTarget);
});
