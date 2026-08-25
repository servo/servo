// This file was procedurally generated from the following sources:
// - src/declarations/var.case
// - src/declarations/redeclare-allow-var/block-attempt-to-redeclare-var-declaration.template
/*---
description: redeclaration with VariableDeclaration (VariableDeclaration in BlockStatement)
esid: sec-block-static-semantics-early-errors
flags: [generated]
info: |
    Block : { StatementList }

    It is a Syntax Error if any element of the LexicallyDeclaredNames of
    StatementList also occurs in the VarDeclaredNames of StatementList.
---*/


{ var f; var f }
