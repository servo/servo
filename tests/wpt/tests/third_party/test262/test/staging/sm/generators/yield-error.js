// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var BUGNUMBER = 1384299;
var summary = "yield outside of generators should provide better error";

assert.throws(
    SyntaxError,
    () => eval("yield 10"),
    "yield expression is only valid in generators");

assert.throws(
    SyntaxError,
    () => eval("yield 10"),
    "yield expression is only valid in generators");

assert.throws(
    SyntaxError,
    () => eval("yield 10"),
    "yield expression is only valid in generators");

