// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  [[Set]] to existing data property of non-extensible `target` is successful.
info: |
  OrdinaryPreventExtensions ( O )

  1. Set O.[[Extensible]] to false.

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
      iii. Let valueDesc be the PropertyDescriptor { [[Value]]: V }.
      iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).

  ValidateAndApplyPropertyDescriptor ( O, P, extensible, Desc, current )

  [...]
  9. If O is not undefined, then
    a. For each field of Desc that is present, set the corresponding attribute
       of the property named P of object O to the value of the field.
  10. Return true.
features: [Symbol]
---*/

var target1 = Object.preventExtensions({ foo: 1 });

Object.assign(target1, { foo: 2 });
assert.sameValue(target1.foo, 2);


var sym = Symbol();
var target2 = { [sym]: 1 };

Object.preventExtensions(target2);
Object.assign(target2, { [sym]: 2 });
assert.sameValue(target2[sym], 2);
