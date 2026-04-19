// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Foundation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  [[Prototype]] defaults to %SharedArrayBufferPrototype% if NewTarget.prototype is not an object.
info: |
  SharedArrayBuffer( length )

  SharedArrayBuffer called with argument length performs the following steps:

  ...
  3. Return AllocateSharedArrayBuffer(NewTarget, byteLength).

  AllocateSharedArrayBuffer( constructor, byteLength )
    1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%SharedArrayBufferPrototype%",
       «[[ArrayBufferData]], [[ArrayBufferByteLength]]» ).
    ...
features: [SharedArrayBuffer, Symbol, Reflect.construct]
---*/

function newTarget() {}

newTarget.prototype = undefined;
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), SharedArrayBuffer.prototype, "newTarget.prototype is undefined");

newTarget.prototype = null;
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [2], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), SharedArrayBuffer.prototype, "newTarget.prototype is null");

newTarget.prototype = true;
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [3], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), SharedArrayBuffer.prototype, "newTarget.prototype is a Boolean");

newTarget.prototype = "";
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [4], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), SharedArrayBuffer.prototype, "newTarget.prototype is a String");

newTarget.prototype = Symbol();
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [5], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), SharedArrayBuffer.prototype, "newTarget.prototype is a Symbol");

newTarget.prototype = 1;
var arrayBuffer = Reflect.construct(SharedArrayBuffer, [6], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), SharedArrayBuffer.prototype, "newTarget.prototype is a Number");
