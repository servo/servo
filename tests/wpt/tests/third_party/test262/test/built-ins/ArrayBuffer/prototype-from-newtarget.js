// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  The [[Prototype]] internal slot is computed from NewTarget.
info: |
  ArrayBuffer( length )

  ArrayBuffer called with argument length performs the following steps:

  ...
  6. Return AllocateArrayBuffer(NewTarget, byteLength).

  AllocateArrayBuffer( constructor, byteLength )
    1. Let obj be OrdinaryCreateFromConstructor(constructor, "%ArrayBufferPrototype%",
       «[[ArrayBufferData]], [[ArrayBufferByteLength]]» ).
    2. ReturnIfAbrupt(obj).
    ...
features: [Reflect.construct]
---*/

var arrayBuffer = Reflect.construct(ArrayBuffer, [8], Object);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), Object.prototype, "NewTarget is built-in Object constructor");

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get: function() {
    return Array.prototype;
  }
});
var arrayBuffer = Reflect.construct(ArrayBuffer, [16], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), Array.prototype, "NewTarget is BoundFunction with accessor");
