// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.entries
description: >
  The method should return an Iterator instance.
info: |
  22.1.3.4 Array.prototype.entries ( )

  1. Let O be ToObject(this value).
  2. ReturnIfAbrupt(O).
  3. Return CreateArrayIterator(O, "key+value").

  22.1.5.1 CreateArrayIterator Abstract Operation

  ...
  2. Let iterator be ObjectCreate(%ArrayIteratorPrototype%, «‍[[IteratedObject]],
  [[ArrayIteratorNextIndex]], [[ArrayIterationKind]]»).
  ...
  6. Return iterator.
features: [Symbol.iterator]
---*/

var ArrayIteratorProto = Object.getPrototypeOf([][Symbol.iterator]());
var iter = [].entries();

assert.sameValue(
  Object.getPrototypeOf(iter), ArrayIteratorProto,
  'The prototype of [].entries() is %ArrayIteratorPrototype%'
);
