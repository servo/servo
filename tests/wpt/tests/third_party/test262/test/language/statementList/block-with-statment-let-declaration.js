// This file was procedurally generated from the following sources:
// - src/statementList/let-declaration.case
// - src/statementList/default/block-with-statement.template
/*---
description: LexicalDeclaration using Let (Valid syntax of StatementList starting with a BlockStatement)
esid: prod-StatementList
flags: [generated]
info: |
    StatementList:
      StatementListItem
      StatementList StatementListItem

    StatementListItem:
      Statement
      Declaration

    Statement:
      BlockStatement

    BlockStatement:
      Block

    Block:
      { StatementList_opt }

    Declaration:
      LexicalDeclaration

    LexicalDeclaration:
      LetOrConst BindingList ;

    BindingList:
      LexicalBinding
      BindingList , LexicalBinding
---*/


// length is a label!
{length: 3000}let a, b = 42, c;b;
