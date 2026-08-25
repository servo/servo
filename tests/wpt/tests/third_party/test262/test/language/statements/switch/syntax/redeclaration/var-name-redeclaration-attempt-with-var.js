// This file was procedurally generated from the following sources:
// - src/declarations/var.case
// - src/declarations/redeclare-allow-var/switch-attempt-to-redeclare-var-declaration.template
/*---
description: redeclaration with VariableDeclaration (VariableDeclaration in SwitchStatement)
esid: sec-switch-statement-static-semantics-early-errors
flags: [generated]
info: |
    SwitchStatement : switch ( Expression ) CaseBlock

    It is a Syntax Error if any element of the LexicallyDeclaredNames of
    CaseBlock also occurs in the VarDeclaredNames of CaseBlock.
---*/


switch (0) { case 1: var f; default: var f }
