// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-block-static-semantics-early-errors
description: >
  Redeclaration with VariableDeclaration (FunctionDeclaration in BlockStatement)
info: |
  13.2.1 Static Semantics: Early Errors

  It is a Syntax Error if any element of the LexicallyDeclaredNames of
  StatementList also occurs in the VarDeclaredNames of StatementList.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function g() {
    // Create an outer block-statement.
    {
        // A lexically declared function declaration.
        function f() {}

        // An inner block-statement with a variable-declared name.
        {
            var f;
        }
    }
}
