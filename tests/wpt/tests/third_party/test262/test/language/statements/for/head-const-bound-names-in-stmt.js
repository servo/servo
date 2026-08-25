// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The body may not re-declare variables declared in the head
info: |
    IterationStatement :
        for ( LexicalDeclaration Expressionopt ; Expressionopt ) Statement

    It is a Syntax Error if any element of the BoundNames of LexicalDeclaration
    also occurs in the VarDeclaredNames of Statement.
negative:
  phase: parse
  type: SyntaxError
esid: sec-for-statement
es6id: 13.7.4
---*/

$DONOTEVALUATE();

for (const x = 0; false; ) {
  var x;
}
