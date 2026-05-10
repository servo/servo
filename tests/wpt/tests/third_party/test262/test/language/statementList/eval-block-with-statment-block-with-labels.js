// This file was procedurally generated from the following sources:
// - src/statementList/block-with-labels.case
// - src/statementList/default/eval-block-with-statement.template
/*---
description: Block with a label (Evaluate produciton of StatementList starting with a BlockStatement)
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

    Statement:
      BlockStatement
      VariableStatement
      EmptyStatement
      ExpressionStatement
      ...

    // lookahead here prevents capturing an Object literal
    ExpressionStatement:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;
---*/


// length is a label!
var result = eval('{length: 3000}{x: 42};');

// Reuse this value for items with empty completions
var expected = 3000;


