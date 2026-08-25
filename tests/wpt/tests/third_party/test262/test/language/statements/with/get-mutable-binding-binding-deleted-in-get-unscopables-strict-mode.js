// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-getbindingvalue-n-s
description: >
  Binding deleted when retrieving unscopables.
info: |
  9.1.1.2.6 GetBindingValue ( N, S )

  1. Let bindingObject be envRec.[[BindingObject]].
  2. Let value be ? HasProperty(bindingObject, N).
  3. If value is false, then
    a. If S is false, return undefined; otherwise throw a ReferenceError exception.
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

var result = null;
with (env) {
  assert.throws(ReferenceError, function() {
    "use strict";
    result = binding;
  });
}

assert.sameValue(unscopablesCalled, 1, "get [Symbol.unscopables] called once");

assert.sameValue(result, null, "result not modified");
