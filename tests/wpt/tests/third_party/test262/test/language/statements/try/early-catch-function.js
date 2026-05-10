// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-try-statement-static-semantics-early-errors
description: >
  Redeclaration of CatchParameter with directly nested FunctionDeclaration in function context.
info: |
  13.15.1 Static Semantics: Early Errors

  It is a Syntax Error if any element of the BoundNames of CatchParameter also
  occurs in the LexicallyDeclaredNames of Block.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function f() {
    try {
    } catch (e) {
        function e(){}
    }
}
