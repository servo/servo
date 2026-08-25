// This file was procedurally generated from the following sources:
// - src/declarations/async-generator.case
// - src/declarations/redeclare/block-attempt-to-redeclare-var-declaration.template
/*---
description: redeclaration with AsyncGeneratorDeclaration (VariableDeclaration in BlockStatement)
esid: sec-block-static-semantics-early-errors
features: [async-iteration]
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

{ var f; async function* f() {} }
