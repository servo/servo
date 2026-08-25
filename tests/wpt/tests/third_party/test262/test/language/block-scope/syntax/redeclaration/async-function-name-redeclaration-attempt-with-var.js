// This file was procedurally generated from the following sources:
// - src/declarations/var.case
// - src/declarations/redeclare-allow-var/block-attempt-to-redeclare-async-function-declaration.template
/*---
description: redeclaration with VariableDeclaration (AsyncFunctionDeclaration in BlockStatement)
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

{ async function f() {} var f }
