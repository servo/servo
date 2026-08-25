// This file was procedurally generated from the following sources:
// - src/declarations/generator.case
// - src/declarations/redeclare/block-attempt-to-redeclare-class-declaration.template
/*---
description: redeclaration with GeneratorDeclaration (ClassDeclaration in BlockStatement)
esid: sec-block-static-semantics-early-errors
features: [generators]
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

{ class f {} function* f() {} }
