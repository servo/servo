// This file was procedurally generated from the following sources:
// - src/statementList/regexp-literal-flags.case
// - src/statementList/default/class-declaration.template
/*---
description: Regular Expression Literal with Flags (Valid syntax of StatementList starting with a Class Declaration)
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

    ExpressionStatement[Yield, Await]:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;

    RegularExpressionLiteral ::
      / RegularExpressionBody / RegularExpressionFlags
---*/


class C {}/1/g;
