// This file was procedurally generated from the following sources:
// - src/statementList/array-literal-with-item.case
// - src/statementList/default/eval-block-with-statement.template
/*---
description: Array Literal with items (Evaluate produciton of StatementList starting with a BlockStatement)
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

    ExpressionStatement:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;

    ArrayLiteral[Yield, Await]:
      [ Elision_opt ]
      [ ElementList ]
      [ ElementList , Elision_opt ]
---*/


// length is a label!
var result = eval('{length: 3000}[42];');

// Reuse this value for items with empty completions
var expected = 3000;


