// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-left-hand-side-expressions-static-semantics-early-errors
description: >
  An Syntax Error is thrown when the syntactic goal symbol is FunctionBody or FormalParameters.
info: |
  It is an early Syntax Error if Module is not the syntactic goal symbol.
features: [import.meta]
---*/

assert.throws(SyntaxError, function() {
    Function("import.meta");
}, "import.meta in FunctionBody");

assert.throws(SyntaxError, function() {
    Function("a = import.meta", "");
}, "import.meta in FormalParameters");
