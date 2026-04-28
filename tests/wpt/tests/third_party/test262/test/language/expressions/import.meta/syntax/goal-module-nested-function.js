// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-left-hand-side-expressions-static-semantics-early-errors
description: >
  No SyntaxError is thrown when import.meta appears in nested functions in module scripts.
info: |
  It is an early Syntax Error if Module is not the syntactic goal symbol.
flags: [module]
features: [import.meta]
---*/

function f() {
  import.meta;
}
