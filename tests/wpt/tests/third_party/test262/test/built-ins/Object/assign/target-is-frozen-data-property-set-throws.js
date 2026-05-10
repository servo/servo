// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  [[Set]] to data property of frozen `target` fails with TypeError.
info: |
  SetIntegrityLevel ( O, level )

  [...]
  3. Let status be ? O.[[PreventExtensions]]().
  [...]
  7. Else,
    a. Assert: level is frozen.
    b. For each element k of keys, do
      i. Let currentDesc be ? O.[[GetOwnProperty]](k).
      ii. If currentDesc is not undefined, then
        1. If IsAccessorDescriptor(currentDesc) is true, then
          [...]
        2. Else,
          a. Let desc be the PropertyDescriptor { [[Configurable]]: false, [[Writable]]: false }.
        3. Perform ? DefinePropertyOrThrow(O, k, desc).
  8. Return true.

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
    a. If ownDesc.[[Writable]] is false, return false.
features: [Symbol, Reflect]
---*/

var sym = Symbol();
var target1 = { [sym]: 1 };

Object.freeze(target1);
assert.throws(TypeError, function() {
  Object.assign(target1, { [sym]: 1 });
});


var target2 = Object.freeze({ foo: 1 });

assert.throws(TypeError, function() {
  Object.assign(target2, { foo: 1 });
});
