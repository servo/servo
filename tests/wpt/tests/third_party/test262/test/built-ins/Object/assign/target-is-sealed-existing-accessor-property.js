// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  [[Set]] to existing accessor property of sealed `target` is successful.
info: |
  SetIntegrityLevel ( O, level )

  [...]
  3. Let status be ? O.[[PreventExtensions]]().
  [...]

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
  7. Perform ? Call(setter, Receiver, « V »).
  8. Return true.
---*/

var value1 = 1;
var target1 = Object.seal({
  set foo(val) { value1 = val; },
});

Object.assign(target1, { foo: 2 });
assert.sameValue(value1, 2);


var sym = Symbol();
var value2 = 1;
var target2 = {
  set [sym](val) { value2 = val; },
};

Object.seal(target2);
Object.assign(target2, { [sym]: 2 });
assert.sameValue(value2, 2);
