// This file was procedurally generated from the following sources:
// - src/statementList/regexp-literal.case
// - src/statementList/default/eval-class-declaration.template
/*---
description: Regular Expression Literal (Valid syntax of StatementList starting with a Class Declaration)
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


var result = eval('class C {}/1/;');

assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.flags, '');
assert.sameValue(result.toString(), '/1/');
