// This file was procedurally generated from the following sources:
// - src/declarations/const.case
// - src/declarations/redeclare/block-attempt-to-redeclare-class-declaration.template
/*---
description: redeclaration with const-LexicalDeclaration (ClassDeclaration in BlockStatement)
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

{ class f {} const f = 0 }
