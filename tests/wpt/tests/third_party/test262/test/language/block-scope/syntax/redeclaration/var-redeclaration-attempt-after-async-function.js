// This file was procedurally generated from the following sources:
// - src/declarations/async-function.case
// - src/declarations/redeclare/block-attempt-to-redeclare-var-declaration-after.template
/*---
description: redeclaration with AsyncFunctionDeclaration (VariableDeclaration in BlockStatement)
esid: sec-block-static-semantics-early-errors
features: [async-functions]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Block : { StatementList }

    It is a Syntax Error if any element of the LexicallyDeclaredNames of
    StatementList also occurs in the VarDeclaredNames of StatementList.

---*/


$DONOTEVALUATE();

{ async function f() {}; var f; }
