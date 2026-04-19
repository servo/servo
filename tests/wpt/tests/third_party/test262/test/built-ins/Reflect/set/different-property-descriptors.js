// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Return false if target property turns to a data descriptor and receiver
  property is an accessor descriptor.
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  4. If receiver is not present, then
    a. Let receiver be target.
  5. Return target.[[Set]](key, V, receiver).

  9.1.9 [[Set]] ( P, V, Receiver)

  ...
  4. If ownDesc is undefined, then
    a. Let parent be O.[[GetPrototypeOf]]().
    b. ReturnIfAbrupt(parent).
    c. If parent is not null, then
      i. Return parent.[[Set]](P, V, Receiver).
    d. Else,
      ii. Let ownDesc be the PropertyDescriptor{[[Value]]: undefined,
      [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true}.
  5. If IsDataDescriptor(ownDesc) is true, then
    a. If ownDesc.[[Writable]] is false, return false.
    b. If Type(Receiver) is not Object, return false.
    c. Let existingDescriptor be Receiver.[[GetOwnProperty]](P).
    d. ReturnIfAbrupt(existingDescriptor).
    e. If existingDescriptor is not undefined, then
      i. If IsAccessorDescriptor(existingDescriptor) is true, return false.
  ...
features: [Reflect, Reflect.set]
---*/

var receiver = {};
var fn = function() {};
Object.defineProperty(receiver, 'p', {
  set: fn
});

var o1 = {};
var result = Reflect.set(o1, 'p', 42, receiver);
assert.sameValue(
  result, false,
  'target has no own `p` and receiver.p has an accessor descriptor'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(receiver, 'p').set, fn,
  'receiver.p is not changed'
);
assert.sameValue(o1.hasOwnProperty('p'), false, 'target.p is not set');

var o2 = {
  p: 43
};
result = Reflect.set(o2, 'p', 42, receiver);
assert.sameValue(
  result, false,
  'target.p has a data descriptor and receiver.p has an accessor descriptor'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(receiver, 'p').set, fn,
  'receiver.p is not changed'
);
assert.sameValue(o2.p, 43, 'target.p is not changed');
