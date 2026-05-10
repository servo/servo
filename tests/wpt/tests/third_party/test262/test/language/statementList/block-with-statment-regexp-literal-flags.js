// This file was procedurally generated from the following sources:
// - src/statementList/regexp-literal-flags.case
// - src/statementList/default/block-with-statement.template
/*---
description: Regular Expression Literal with Flags (Valid syntax of StatementList starting with a BlockStatement)
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

    ExpressionStatement[Yield, Await]:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;

    RegularExpressionLiteral ::
      / RegularExpressionBody / RegularExpressionFlags
---*/


// length is a label!
{length: 3000}/1/g;
