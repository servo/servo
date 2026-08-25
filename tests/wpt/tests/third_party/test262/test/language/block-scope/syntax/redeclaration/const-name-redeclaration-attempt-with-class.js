// This file was procedurally generated from the following sources:
// - src/declarations/class.case
// - src/declarations/redeclare/block-attempt-to-redeclare-const-declaration.template
/*---
description: redeclaration with ClassDeclaration (LexicalDeclaration (const) in BlockStatement)
esid: sec-block-static-semantics-early-errors
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Block : { StatementList }

    It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains
    any duplicate entries.

---*/


$DONOTEVALUATE();

{ const f = 0; class f {} }
