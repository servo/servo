// This file was procedurally generated from the following sources:
// - src/statementList/let-declaration.case
// - src/statementList/default/function-declaration.template
/*---
description: LexicalDeclaration using Let (Valid syntax of StatementList starting with a Function Declaration)
esid: prod-StatementList
flags: [generated]
info: |
    StatementList:
      StatementListItem
      StatementList StatementListItem

    StatementListItem:
      Statement
      Declaration

    Declaration:
      HoistableDeclaration

    FunctionDeclaration:
      function BindingIdentifier ( FormalParameters ) { FunctionBody }

    Declaration:
      LexicalDeclaration

    LexicalDeclaration:
      LetOrConst BindingList ;

    BindingList:
      LexicalBinding
      BindingList , LexicalBinding
---*/


function fn() {}let a, b = 42, c;b;
