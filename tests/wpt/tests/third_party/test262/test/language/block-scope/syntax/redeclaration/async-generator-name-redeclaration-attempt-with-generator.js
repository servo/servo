// This file was procedurally generated from the following sources:
// - src/declarations/generator.case
// - src/declarations/redeclare/block-attempt-to-redeclare-async-generator-declaration.template
/*---
description: redeclaration with GeneratorDeclaration (AsyncGeneratorDeclaration in BlockStatement)
esid: sec-block-static-semantics-early-errors
features: [generators, async-iteration]
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

{ async function* f() {} function* f() {} }
