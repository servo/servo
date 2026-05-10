// This file was procedurally generated from the following sources:
// - src/statementList/array-literal-with-item.case
// - src/statementList/default/eval-class-declaration.template
/*---
description: Array Literal with items (Valid syntax of StatementList starting with a Class Declaration)
esid: prod-StatementList
features: [class]
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

    ArrayLiteral[Yield, Await]:
      [ Elision_opt ]
      [ ElementList ]
      [ ElementList , Elision_opt ]
---*/


var result = eval('class C {}[42];');

assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert.sameValue(result.length, 1);
assert.sameValue(result[0], 42);
