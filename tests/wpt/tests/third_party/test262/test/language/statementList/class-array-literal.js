// This file was procedurally generated from the following sources:
// - src/statementList/array-literal.case
// - src/statementList/default/class-declaration.template
/*---
description: Array Literal (Valid syntax of StatementList starting with a Class Declaration)
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


class C {}[];
