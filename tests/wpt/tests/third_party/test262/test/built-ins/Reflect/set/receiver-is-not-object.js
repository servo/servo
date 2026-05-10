// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Return false if receiver is not an object.
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
  ...
features: [Reflect, Reflect.set]
---*/

var o1 = {
  p: 42
};
var receiver = 'receiver is a string';
var result = Reflect.set(o1, 'p', 43, receiver);

assert.sameValue(result, false, 'returns false');
assert.sameValue(o1.p, 42, 'does not set a value');
assert.sameValue(
  receiver.hasOwnProperty('p'), false,
  'does not set a new property on receiver if it is not an object'
);
