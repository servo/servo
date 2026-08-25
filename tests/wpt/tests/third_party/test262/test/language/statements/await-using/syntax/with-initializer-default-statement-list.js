// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations-static-semantics-early-errors
description: >
    await using declarations with initialisers in statement positions:
    default : StatementList
info: |
  AwaitUsingDeclaration : CoverAwaitExpressionAndAwaitUsingDeclarationHead BindingList ;

  - It is a Syntax Error if AwaitUsingDeclaration is contained directly within the StatementList of either a CaseClause or
    DefaultClause.

negative:
  phase: parse
  type: SyntaxError
features: [explicit-resource-management]
---*/

$DONOTEVALUATE();
async function f() {
    switch (true) { default: await using x = null; }
}
