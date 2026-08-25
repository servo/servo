// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  The [[Prototype]] internal slot is computed from NewTarget.
info: |
  SharedArrayBuffer( length )

  SharedArrayBuffer called with argument length performs the following steps:

  ...
  3. Return AllocateSharedArrayBuffer(NewTarget, byteLength).

  AllocateSharedArrayBuffer( constructor, byteLength )
    1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%SharedArrayBufferPrototype%",
       «[[ArrayBufferData]], [[ArrayBufferByteLength]]» ).
    ...
features: [Reflect, Reflect.construct, SharedArrayBuffer]
---*/

var arrayBuffer = Reflect.construct(SharedArrayBuffer, [8], Object);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), Object.prototype, "NewTarget is built-in Object constructor");

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get: function() {
    return Array.prototype;
  }
});
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [16], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), Array.prototype, "NewTarget is BoundFunction with accessor");
