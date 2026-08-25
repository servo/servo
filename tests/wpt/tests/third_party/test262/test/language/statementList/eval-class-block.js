// This file was procedurally generated from the following sources:
// - src/statementList/block.case
// - src/statementList/default/eval-class-declaration.template
/*---
description: Block (Valid syntax of StatementList starting with a Class Declaration)
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

    // lookahead here prevents capturing an Object literal
    ExpressionStatement:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;
---*/


var result = eval('class C {}{}');

assert.sameValue(result, undefined);
