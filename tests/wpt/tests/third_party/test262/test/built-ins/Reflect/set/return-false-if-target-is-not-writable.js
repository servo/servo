// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Return false if target is not writable.
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  4. If receiver is not present, then
    a. Let receiver be target.
  5. Return target.[[Set]](key, V, receiver).

  9.1.9 [[Set]] ( P, V, Receiver)

  ...
  5. If IsDataDescriptor(ownDesc) is true, then
    a. If ownDesc.[[Writable]] is false, return false.
    b. If Type(Receiver) is not Object, return false.
    c. Let existingDescriptor be Receiver.[[GetOwnProperty]](P).
    d. ReturnIfAbrupt(existingDescriptor).
    e. If existingDescriptor is not undefined, then
      i. If IsAccessorDescriptor(existingDescriptor) is true, return false.
      ii. If existingDescriptor.[[Writable]] is false, return false.
  ...
features: [Reflect, Reflect.set]
---*/

var o1 = {};
var receiver = {};
Object.defineProperty(receiver, 'p', {
  writable: false,
  value: 42
});
var result = Reflect.set(o1, 'p', 43, receiver);
assert.sameValue(result, false, 'returns false');
assert.sameValue(receiver.p, 42, 'does not set a new value on receiver');
assert.sameValue(
  o1.hasOwnProperty('p'), false,
  'does not set a new value on target'
);
