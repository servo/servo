// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-left-hand-side-expressions-static-semantics-early-errors
description: >
  An Syntax Error is thrown when the syntactic goal symbol is AsyncFunctionBody or FormalParameters.
info: |
  It is an early Syntax Error if Module is not the syntactic goal symbol.
features: [import.meta, async-functions]
---*/

var AsyncFunction = async function(){}.constructor;

assert.throws(SyntaxError, function() {
    AsyncFunction("import.meta");
}, "import.meta in AsyncFunctionBody");

assert.throws(SyntaxError, function() {
    AsyncFunction("a = import.meta", "");
}, "import.meta in FormalParameters");
