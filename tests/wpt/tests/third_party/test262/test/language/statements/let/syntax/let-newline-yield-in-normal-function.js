// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations
description: >
  `let yield` does not permit ASI in between, as `yield` is a BindingIdentifier
info: |
  `yield` is a perfectly cromulent binding name in any context grammatically, just
  prohibited by static semantics in some contexts.  Therefore ASI can never apply
  between `let` (where a LexicalDeclaration is permitted) and `yield`,
  so a subsequent `0` where `=` was expected is a syntax error.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function f() {
    let
    yield 0;
}
