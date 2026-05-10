// This file was procedurally generated from the following sources:
// - src/declarations/var.case
// - src/declarations/redeclare-allow-var/switch-attempt-to-redeclare-async-function-declaration.template
/*---
description: redeclaration with VariableDeclaration (AsyncFunctionDeclaration in SwitchStatement)
esid: sec-switch-statement-static-semantics-early-errors
features: [async-functions]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    SwitchStatement : switch ( Expression ) CaseBlock

    It is a Syntax Error if any element of the LexicallyDeclaredNames of
    CaseBlock also occurs in the VarDeclaredNames of CaseBlock.

---*/


$DONOTEVALUATE();

switch (0) { case 1: async function f() {} default: var f }
