// This file was procedurally generated from the following sources:
// - src/declarations/async-function.case
// - src/declarations/redeclare/block-attempt-to-redeclare-let-declaration.template
/*---
description: redeclaration with AsyncFunctionDeclaration (LexicalDeclaration (let) in BlockStatement)
esid: sec-block-static-semantics-early-errors
features: [async-functions]
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

{ let f; async function f() {} }
