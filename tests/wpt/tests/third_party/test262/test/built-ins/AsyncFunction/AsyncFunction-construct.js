// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-constructor
description: >
  %AsyncFunction% creates functions with or without new and handles arguments
  similarly to functions.
---*/

var AsyncFunction = async function foo() {}.constructor;
var fn;

fn = AsyncFunction("a", "await 1;");
assert.sameValue(fn.length, 1, "length with 1 argument, call");

fn = AsyncFunction("a,b", "await 1;");
assert.sameValue(fn.length, 2, "length with 2 arguments in one, call");

fn = AsyncFunction("a", "b", "await 1;");
assert.sameValue(fn.length, 2, "length with 2 arguments, call");

fn = new AsyncFunction("a", "await 1;");
assert.sameValue(fn.length, 1, "length with 1 argument, construct");

fn = new AsyncFunction("a,b", "await 1;");
assert.sameValue(fn.length, 2, "length with 2 arguments in one, construct");

fn = new AsyncFunction("a", "b", "await 1;");
assert.sameValue(fn.length, 2, "length with 2 arguments, construct");
