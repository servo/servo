// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Creates a property data descriptor.
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
      ii. If existingDescriptor.[[Writable]] is false, return false.
      iii. Let valueDesc be the PropertyDescriptor{[[Value]]: V}.
      iv. Return Receiver.[[DefineOwnProperty]](P, valueDesc).
    f. Else Receiver does not currently have a property P,
      i. Return CreateDataProperty(Receiver, P, V).
  6. Assert: IsAccessorDescriptor(ownDesc) is true.
  7. Let setter be ownDesc.[[Set]].
  8. If setter is undefined, return false.
  ...
  11. Return true.
includes: [propertyHelper.js]
features: [Reflect, Reflect.set]
---*/

var o1 = {};
var result = Reflect.set(o1, 'p', 42);
assert.sameValue(result, true, 'returns true on a successful setting');
var desc = Object.getOwnPropertyDescriptor(o1, 'p');
assert.sameValue(
  desc.value, 42,
  'sets a data descriptor to set a new property'
);
verifyProperty(o1, 'p', {
  writable: true,
  enumerable: true,
  configurable: true,
});

var o2 = {};
var receiver = {};
result = Reflect.set(o2, 'p', 43, receiver);
assert.sameValue(
  result, true,
  'returns true on a successful setting with a receiver'
);
desc = Object.getOwnPropertyDescriptor(o2, 'p');
assert.sameValue(
  desc, undefined,
  'does not set a data descriptor on target if receiver is given'
);
desc = Object.getOwnPropertyDescriptor(receiver, 'p');
assert.sameValue(
  desc.value, 43,
  'sets a data descriptor on the receiver object to set a new property'
);
verifyProperty(receiver, 'p', {
  writable: true,
  enumerable: true,
  configurable: true,
});
