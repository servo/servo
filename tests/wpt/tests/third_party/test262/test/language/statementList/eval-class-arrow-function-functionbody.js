// This file was procedurally generated from the following sources:
// - src/statementList/arrow-function-functionbody.case
// - src/statementList/default/eval-class-declaration.template
/*---
description: Arrow Function with a Function Body (Valid syntax of StatementList starting with a Class Declaration)
esid: prod-StatementList
features: [arrow-function, class]
flags: [generated]
info: |
    StatementList:
      StatementListItem
      StatementList StatementListItem

    StatementListItem:
      Statement
      Declaration

    Declaration:
      ClassDeclaration


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


var result = eval('class C {}() => { return 42; };');

assert.sameValue(result(), 42);
