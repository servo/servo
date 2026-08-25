// This file was procedurally generated from the following sources:
// - src/statementList/arrow-function-assignment-expr.case
// - src/statementList/default/function-declaration.template
/*---
description: Arrow Function with an AssignmentExpression (Valid syntax of StatementList starting with a Function Declaration)
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

    Declaration:
      HoistableDeclaration

    FunctionDeclaration:
      function BindingIdentifier ( FormalParameters ) { FunctionBody }

    Statement:
      BlockStatement
      VariableStatement
      EmptyStatement
      ExpressionStatement
      ...

    ExpressionStatement:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;

    ...

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


function fn() {}() => 42;
