// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  [[Set]] to non-existing property of non-extensible `target` fails with TypeError.
info: |
  Object.assign ( target, ...sources )

  [...]
  3. For each element nextSource of sources, do
    a. If nextSource is neither undefined nor null, then
      [...]
      iii. For each element nextKey of keys, do
        1. Let desc be ? from.[[GetOwnProperty]](nextKey).
        2. If desc is not undefined and desc.[[Enumerable]] is true, then
          [...]
          b. Perform ? Set(to, nextKey, propValue, true).

  OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

  [...]
  3. If IsDataDescriptor(ownDesc) is true, then
    [...]
    c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
    d. If existingDescriptor is not undefined, then
      [...]
    e. Else,
      i. Assert: Receiver does not currently have a property P.
      ii. Return ? CreateDataProperty(Receiver, P, V).

  ValidateAndApplyPropertyDescriptor ( O, P, extensible, Desc, current )

  [...]
  2. If current is undefined, then
    a. If extensible is false, return false.
features: [Symbol]
---*/

var target1 = Object.preventExtensions({ foo: 1 });

assert.throws(TypeError, function() {
  Object.assign(target1, { get bar() {} });
});


var target2 = {};

Object.preventExtensions(target2);
assert.throws(TypeError, function() {
  Object.assign(target2, { [Symbol()]: 1 });
});
