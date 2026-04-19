// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  The new ArrayBuffer instance is created prior to allocating the Data Block.
info: |
  ArrayBuffer ( length [ , options ] )

  ...
  4. Return ? AllocateArrayBuffer(NewTarget, byteLength, requestedMaxByteLength).

  AllocateArrayBuffer ( constructor, byteLength [ , maxByteLength ] )

  ...
  4. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", slots).
  5. Let block be ? CreateByteDataBlock(byteLength).
  ...

features: [resizable-arraybuffer, Reflect.construct]
---*/

function DummyError() {}

let newTarget = Object.defineProperty(function(){}.bind(null), "prototype", {
  get() {
    throw new DummyError();
  }
});

assert.throws(DummyError, function() {
  let byteLength = 0;
  let options = {
    maxByteLength: 7 * 1125899906842624
  };

  // Allocating 7 PiB should fail with a RangeError.
  // Math.pow(1024, 5) = 1125899906842624
  Reflect.construct(ArrayBuffer, [], newTarget);
});
