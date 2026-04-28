// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  [[Set]] to accessor property of frozen `target` succeeds.
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
          a. Let desc be the PropertyDescriptor { [[Configurable]]: false }.
        [...]
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
  7. Perform ? Call(setter, Receiver, « V »).
  8. Return true.
features: [Symbol]
---*/

var value1 = 1;
var target1 = {
  set foo(val) { value1 = val; },
};

Object.freeze(target1);
Object.assign(target1, { foo: 2 });
assert.sameValue(value1, 2);


var sym = Symbol();
var value2 = 1;
var target2 = Object.freeze({
  set [sym](val) { value2 = val; },
});

Object.freeze(target2);
Object.assign(target2, { [sym]: 2 });
assert.sameValue(value2, 2);
