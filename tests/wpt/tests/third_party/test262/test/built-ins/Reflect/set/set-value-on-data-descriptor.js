// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Sets the new value.
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
      iii. Let valueDesc be the PropertyDescriptor{[[Value]]: V}.
      iv. Return Receiver.[[DefineOwnProperty]](P, valueDesc).
    f. Else Receiver does not currently have a property P,
      i. Return CreateDataProperty(Receiver, P, V).
  ...
features: [Reflect, Reflect.set]
---*/

var o1 = {
  p: 43
};
var result = Reflect.set(o1, 'p', 42);
assert.sameValue(result, true, 'returns true on a successful setting');
assert.sameValue(
  o1.p, 42,
  'sets the new value'
);

var o2 = {
  p: 43
};
var receiver = {
  p: 44
};
var result = Reflect.set(o2, 'p', 42, receiver);
assert.sameValue(result, true, 'returns true on a successful setting');
assert.sameValue(o2.p, 43, 'with a receiver, does not set a value on target');
assert.sameValue(receiver.p, 42, 'sets the new value on the receiver');
