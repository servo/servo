// This file was procedurally generated from the following sources:
// - src/statementList/let-declaration.case
// - src/statementList/default/eval-block-with-statement.template
/*---
description: LexicalDeclaration using Let (Evaluate produciton of StatementList starting with a BlockStatement)
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
var result = eval('{length: 3000}let a, b = 42, c;b;');

// Reuse this value for items with empty completions
var expected = 3000;


