// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  demonstrate syntax-based short-circuiting.
info: |
  If the expression on the LHS of ?. evaluates to null/undefined, the RHS is
  not evaluated
features: [optional-chaining]
---*/

const a = undefined;
let x = 1;

a?.[++x] // short-circuiting.
a?.b.c(++x).d; // long short-circuiting.

undefined?.[++x] // short-circuiting.
undefined?.b.c(++x).d; // long short-circuiting.

assert.sameValue(1, x);
