// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-setmutablebinding-n-v-s
description: >
  Binding deleted when retrieving unscopables.
info: |
  9.1.1.2.5 SetMutableBinding ( N, V, S )

  1. Let bindingObject be envRec.[[BindingObject]].
  2. Let stillExists be ? HasProperty(bindingObject, N).
  3. If stillExists is false and S is true, throw a ReferenceError exception.
  ...

flags: [noStrict]
features: [Symbol.unscopables]
---*/

var unscopablesCalled = 0;

var env = {
  binding: 0,
  get [Symbol.unscopables]() {
    unscopablesCalled++;
    delete env.binding;
    return null;
  }
};

with (env) {
  assert.throws(ReferenceError, function() {
    "use strict";
    binding = 123;
  });
}

assert.sameValue(unscopablesCalled, 1, "get [Symbol.unscopables] called once");

assert.sameValue(Object.getOwnPropertyDescriptor(env, "binding"), undefined, "binding deleted");
