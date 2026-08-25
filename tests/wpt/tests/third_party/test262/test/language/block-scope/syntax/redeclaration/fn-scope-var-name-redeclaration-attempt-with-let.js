// This file was procedurally generated from the following sources:
// - src/declarations/let.case
// - src/declarations/redeclare/fn-block-attempt-to-redeclare-var-declaration.template
/*---
description: redeclaration with let-LexicalDeclaration (VariableDeclaration in BlockStatement inside a function)
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

function x() {
  { let f; var f; }
}
