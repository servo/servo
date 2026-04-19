// This file was procedurally generated from the following sources:
// - src/statementList/let-declaration.case
// - src/statementList/default/eval-block.template
/*---
description: LexicalDeclaration using Let (Eval production of StatementList starting with a BlockStatement)
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


var result = eval('{}let a, b = 42, c;b;');

assert.sameValue(result, 42);
