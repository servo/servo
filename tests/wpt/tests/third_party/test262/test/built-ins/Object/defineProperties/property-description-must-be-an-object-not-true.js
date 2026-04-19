// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-objectdefineproperties
description: >
  Property description must be an object (true)
info: |
  ObjectDefineProperties ( O, Properties )

  ...
  For each element nextKey of keys, do
    Let propDesc be ? props.[[GetOwnProperty]](nextKey).
    If propDesc is not undefined and propDesc.[[Enumerable]] is true, then
    Let descObj be ? Get(props, nextKey).
    Let desc be ? ToPropertyDescriptor(descObj).
  ...

  ToPropertyDescriptor ( Obj )

  If Type(Obj) is not Object, throw a TypeError exception.
  ...
---*/

assert.throws(TypeError, () => {
  Object.defineProperties({}, {
    a: true
  });
});
