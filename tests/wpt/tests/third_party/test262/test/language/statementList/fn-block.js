// This file was procedurally generated from the following sources:
// - src/statementList/block.case
// - src/statementList/default/function-declaration.template
/*---
description: Block (Valid syntax of StatementList starting with a Function Declaration)
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

    // lookahead here prevents capturing an Object literal
    ExpressionStatement:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;
---*/


function fn() {}{}
