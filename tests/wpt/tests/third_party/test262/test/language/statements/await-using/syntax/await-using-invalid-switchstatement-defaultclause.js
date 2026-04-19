// Copyright (C) 2025 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations-static-semantics-early-errors
description: Disallowed in switch statement
info: |
  - It is a Syntax Error if the goal symbol is |Script| and |AwaitUsingDeclaration| is not contained, either directly or indirectly, within a |Block|, |ForStatement|, |ForInOfStatement|, |FunctionBody|, |GeneratorBody|, |AsyncGeneratorBody|, |AsyncFunctionBody|, or |ClassStaticBlockBody|.
  - It is a Syntax Error if |AwaitUsingDeclaration| is contained directly within the |StatementList| of either a |CaseClause| or |DefaultClause|.

negative:
  phase: parse
  type: SyntaxError

features: [explicit-resource-management]
---*/

async function f() {
  switch (0) {
    default:
      await using _ = null;
      break;
  }
}

$DONOTEVALUATE();
