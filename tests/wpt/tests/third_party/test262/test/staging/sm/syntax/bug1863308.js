// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.throws(
    SyntaxError,
    () => eval("for (let case of ['foo', 'bar']) {}"),
    "unexpected token: keyword 'case'");

assert.throws(
    SyntaxError,
    () => eval("for (let debugger of ['foo', 'bar']) {}"),
    "unexpected token: keyword 'debugger'");

assert.throws(
    SyntaxError,
    () => eval("for (let case in ['foo', 'bar']) {}"),
    "unexpected token: keyword 'case'");

assert.throws(
    SyntaxError,
    () => eval("for (let debugger in ['foo', 'bar']) {}"),
    "unexpected token: keyword 'debugger'");

