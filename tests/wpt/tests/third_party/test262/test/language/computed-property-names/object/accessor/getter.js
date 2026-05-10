// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    Computed property names for getters
---*/
var s = Symbol();
var A = {
  get ["a"]() {
    return "A";
  },
  get [1]() {
    return 1;
  },
  get [s]() {
    return s;
  }
};
assert.sameValue(A.a, "A", "The value of `A.a` is `'A'`");
assert.sameValue(A[1], 1, "The value of `A[1]` is `1`");
assert.sameValue(A[s], s, "The value of `A[s]` is `Symbol()`");
