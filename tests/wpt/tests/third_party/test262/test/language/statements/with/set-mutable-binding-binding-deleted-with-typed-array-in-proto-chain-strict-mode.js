// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-setmutablebinding-n-v-s
description: >
  Typed Array index binding deleted from object.
info: |
  9.1.1.2.5 SetMutableBinding ( N, V, S )

  1. Let bindingObject be envRec.[[BindingObject]].
  2. Let stillExists be ? HasProperty(bindingObject, N).
  3. If stillExists is false and S is true, throw a ReferenceError exception.
  ...

flags: [noStrict]
features: [TypedArray]
---*/

var typedArray = new Int32Array(10);

var env = Object.create(typedArray);

Object.defineProperty(env, "NaN", {
  configurable: true,
  value: 100,
});

with (env) {
  assert.throws(ReferenceError, function() {
    "use strict";
    NaN = (delete env.NaN, 0);
  });
}

assert.sameValue(Object.getOwnPropertyDescriptor(env, "NaN"), undefined);
