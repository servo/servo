// This file was procedurally generated from the following sources:
// - src/statementList/expr-arrow-function-boolean-literal.case
// - src/statementList/default/eval-block-with-statement.template
/*---
description: Expression with an Arrow Function and Boolean literal (Evaluate produciton of StatementList starting with a BlockStatement)
esid: prod-StatementList
features: [arrow-function]
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

    Expression:
      AssignmentExpression
      Expression , AssignmentExpression

    AssignmentExpression:
      ConditionalExpression
      [+Yield]YieldExpression
      ArrowFunction

    ArrowFunction:
      ArrowParameters [no LineTerminator here] => ConciseBody

    ConciseBody:
      [lookahead ≠ {] AssignmentExpression
      { FunctionBody }

---*/


// length is a label!
var result = eval('{length: 3000}() => 1, 42;');

// Reuse this value for items with empty completions
var expected = 3000;


