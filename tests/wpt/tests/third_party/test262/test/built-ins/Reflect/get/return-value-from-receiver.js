// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.6
description: >
  Return value from a receiver.
info: |
  26.1.6 Reflect.get ( target, propertyKey [ , receiver ])

  ...
  4. If receiver is not present, then
    a. Let receiver be target.
  5. Return target.[[Get]](key, receiver).

  9.1.8 [[Get]] (P, Receiver)

  ...
  2. Let desc be O.[[GetOwnProperty]](P).
  3. ReturnIfAbrupt(desc).
  4. If desc is undefined, then
    a. Let parent be O.[[GetPrototypeOf]]().
    b. ReturnIfAbrupt(parent).
    c. If parent is null, return undefined.
    d. Return parent.[[Get]](P, Receiver).
  5. If IsDataDescriptor(desc) is true, return desc.[[Value]].
  6. Otherwise, IsAccessorDescriptor(desc) must be true so, let getter be
  desc.[[Get]].
  7. If getter is undefined, return undefined.
  8. Return Call(getter, Receiver).
features: [Reflect]
---*/

var o1 = {};
var receiver = {
  y: 42
};

Object.defineProperty(o1, 'x', {
  get: function() {
    return this.y;
  }
});
assert.sameValue(
  Reflect.get(o1, 'x', receiver), 42,
  'Return own property value using a receiver'
);

var o2 = Object.create(o1);
assert.sameValue(
  Reflect.get(o2, 'x', receiver), 42,
  'Return prototype property value using a receiver'
);
