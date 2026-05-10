// This file was procedurally generated from the following sources:
// - src/statementList/regexp-literal.case
// - src/statementList/default/eval-block.template
/*---
description: Regular Expression Literal (Eval production of StatementList starting with a BlockStatement)
esid: prod-StatementList
flags: [generated]
info: |
    StatementList:
      StatementListItem
      StatementList StatementListItem

    StatementListItem:
      Statement
      Declaration

    Statement:
      BlockStatement

    BlockStatement:
      Block

    Block:
      { StatementList_opt }

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


var result = eval('{}/1/;');

assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.flags, '');
assert.sameValue(result.toString(), '/1/');
