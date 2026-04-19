// This file was procedurally generated from the following sources:
// - src/statementList/block-with-labels.case
// - src/statementList/default/eval-class-declaration.template
/*---
description: Block with a label (Valid syntax of StatementList starting with a Class Declaration)
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


var result = eval('class C {}{x: 42};');

assert.sameValue(result, 42, 'it does not evaluate to an Object with the property x');
