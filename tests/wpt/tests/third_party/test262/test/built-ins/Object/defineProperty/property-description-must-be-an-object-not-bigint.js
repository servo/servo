// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.defineproperty
description: >
  Property description must be an object (bigint)
info: |
  Object.defineProperty ( O, P, Attributes )

  ...
  Let desc be ? ToPropertyDescriptor(Attributes).
  ...

  ToPropertyDescriptor ( Obj )

  If Type(Obj) is not Object, throw a TypeError exception.
  ...
features: [BigInt]
---*/

assert.throws(TypeError, () => {
  Object.defineProperty({}, 'a', 0n);
});
