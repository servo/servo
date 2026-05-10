// This file was procedurally generated from the following sources:
// - src/declarations/generator.case
// - src/declarations/redeclare/fn-block-attempt-to-redeclare-var-declaration.template
/*---
description: redeclaration with GeneratorDeclaration (VariableDeclaration in BlockStatement inside a function)
esid: sec-block-static-semantics-early-errors
features: [generators]
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

function x() {
  { function* f() {}; var f; }
}
