// This file was procedurally generated from the following sources:
// - src/statementList/regexp-literal-flags.case
// - src/statementList/default/eval-function-declaration.template
/*---
description: Regular Expression Literal with Flags (Eval production of StatementList starting with a Function Declaration)
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

    ExpressionStatement[Yield, Await]:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression ;

    RegularExpressionLiteral ::
      / RegularExpressionBody / RegularExpressionFlags
---*/


var result = eval('function fn() {}/1/g;');

assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.flags, 'g');
assert.sameValue(result.toString(), '/1/g');
