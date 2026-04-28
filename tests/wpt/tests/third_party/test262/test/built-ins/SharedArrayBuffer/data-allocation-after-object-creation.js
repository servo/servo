// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  The new SharedArrayBuffer instance is created prior to allocating the Data Block.
info: |
  SharedArrayBuffer( length )

  ...
  3. Return AllocateSharedArrayBuffer(NewTarget, byteLength).

  AllocateSharedArrayBuffer( constructor, byteLength )
    1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%SharedArrayBufferPrototype%",
       «[[ArrayBufferData]], [[ArrayBufferByteLength]]» ).
    ...
    3. Let block be ? CreateByteDataBlock(byteLength).
    ...
features: [SharedArrayBuffer, Reflect.construct]
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
  Reflect.construct(SharedArrayBuffer, [7 * 1125899906842624], newTarget);
});
