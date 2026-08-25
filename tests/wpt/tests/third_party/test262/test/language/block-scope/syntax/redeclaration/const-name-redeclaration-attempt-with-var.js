// This file was procedurally generated from the following sources:
// - src/declarations/var.case
// - src/declarations/redeclare-allow-var/block-attempt-to-redeclare-const-declaration.template
/*---
description: redeclaration with VariableDeclaration (LexicalDeclaration (const) in BlockStatement)
esid: sec-block-static-semantics-early-errors
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

{ const f = 0; var f }
