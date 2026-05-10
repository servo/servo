// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.throws(SyntaxError, () => eval(`class A { #x; #x; }`));

// No computed private fields
assert.throws(SyntaxError, () => eval(`var x = "foo"; class A { #[x] = 20; }`));

assert.throws(
    SyntaxError,
    () => eval(`class A { #x; h(o) { return !#x; }}`),
    "invalid use of private name in unary expression without object reference");

assert.throws(
    SyntaxError,
    () => eval(`class A { #x; h(o) { return 1 + #x in o; }}`),
    "invalid use of private name due to operator precedence");


