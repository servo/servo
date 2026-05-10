// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  The byteLength argument is validated before OrdinaryCreateFromConstructor.
info: |
  ArrayBuffer ( length [ , options ] )

  ...
  4. Return ? AllocateArrayBuffer(NewTarget, byteLength, requestedMaxByteLength).

  AllocateArrayBuffer ( constructor, byteLength [ , maxByteLength ] )

  ...
  3. If allocatingResizableBuffer is true, then
    a. If byteLength > maxByteLength, throw a RangeError exception.
  ...
  4. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", slots).
  ...

features: [resizable-arraybuffer, Reflect.construct]
---*/

let newTarget = Object.defineProperty(function(){}.bind(null), "prototype", {
  get() {
    throw new Test262Error();
  }
});

assert.throws(RangeError, function() {
  let byteLength = 10;
  let options = {
    maxByteLength: 0,
  };

  // Throws a RangeError, because `byteLength` is larger than `options.maxByteLength`.
  Reflect.construct(ArrayBuffer, [byteLength, options], newTarget);
});
