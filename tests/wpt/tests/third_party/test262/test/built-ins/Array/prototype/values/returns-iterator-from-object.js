// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.values
description: >
  Creates an iterator from a custom object.
info: |
  22.1.3.29 Array.prototype.values ( )

  1. Let O be ToObject(this value).
  2. ReturnIfAbrupt(O).
  3. Return CreateArrayIterator(O, "value").
features: [Symbol.iterator]
---*/

var obj = {
  length: 2
};
var iter = Array.prototype.values.call(obj);
var ArrayIteratorProto = Object.getPrototypeOf([][Symbol.iterator]());

assert.sameValue(
  Object.getPrototypeOf(iter), ArrayIteratorProto,
  'The prototype of [].values() is %ArrayIteratorPrototype%'
);
