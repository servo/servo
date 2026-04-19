// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  [[Prototype]] defaults to %ArrayBufferPrototype% if NewTarget.prototype is not an object.
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
features: [Reflect.construct, Symbol]
---*/

function newTarget() {}

newTarget.prototype = undefined;
var arrayBuffer = Reflect.construct(ArrayBuffer, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), ArrayBuffer.prototype, "newTarget.prototype is undefined");

newTarget.prototype = null;
var arrayBuffer = Reflect.construct(ArrayBuffer, [2], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), ArrayBuffer.prototype, "newTarget.prototype is null");

newTarget.prototype = true;
var arrayBuffer = Reflect.construct(ArrayBuffer, [3], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), ArrayBuffer.prototype, "newTarget.prototype is a Boolean");

newTarget.prototype = "";
var arrayBuffer = Reflect.construct(ArrayBuffer, [4], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), ArrayBuffer.prototype, "newTarget.prototype is a String");

newTarget.prototype = Symbol();
var arrayBuffer = Reflect.construct(ArrayBuffer, [5], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), ArrayBuffer.prototype, "newTarget.prototype is a Symbol");

newTarget.prototype = 1;
var arrayBuffer = Reflect.construct(ArrayBuffer, [6], newTarget);
assert.sameValue(Object.getPrototypeOf(arrayBuffer), ArrayBuffer.prototype, "newTarget.prototype is a Number");
