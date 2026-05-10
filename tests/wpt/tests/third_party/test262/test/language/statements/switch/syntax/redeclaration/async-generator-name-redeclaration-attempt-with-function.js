// This file was procedurally generated from the following sources:
// - src/declarations/function.case
// - src/declarations/redeclare-allow-sloppy-function/switch-attempt-to-redeclare-async-generator-declaration.template
/*---
description: redeclaration with FunctionDeclaration (AsyncGeneratorDeclaration in SwitchStatement)
esid: sec-switch-statement-static-semantics-early-errors
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    SwitchStatement : switch ( Expression ) CaseBlock

    It is a Syntax Error if the LexicallyDeclaredNames of CaseBlock contains any
    duplicate entries.

---*/


$DONOTEVALUATE();

switch (0) { case 1: async function* f() {} default: function f() {} }
