// This file was procedurally generated from the following sources:
// - src/statementList/array-literal.case
// - src/statementList/default/eval-function-declaration.template
/*---
description: Array Literal (Eval production of StatementList starting with a Function Declaration)
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


var result = eval('function fn() {}[];');

assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert.sameValue(result.length, 0);
