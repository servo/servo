// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-nested.case
// - src/top-level-await/syntax/for-await-expr.template
/*---
description: Nested AwaitExpressions (Valid syntax for top level await in for await statements.)
esid: prod-AwaitExpression
features: [top-level-await, async-iteration]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    IterationStatement[Yield, Await, Return]:
      [+Await]for await ( [lookahead ≠ let] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      [+Await]for await ( var ForBinding[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      [+Await]for await ( ForDeclaration[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]

    ...

    UnaryExpression[Yield, Await]
      [+Await]AwaitExpression[?Yield]

    AwaitExpression[Yield]:
      await UnaryExpression[?Yield, +Await]

    ...


    TryStatement[Yield, Await, Return]:
      try Block[?Yield, ?Await, ?Return] Catch[?Yield, ?Await, ?Return]
      try Block[?Yield, ?Await, ?Return] Finally[?Yield, ?Await, ?Return]
      try Block[?Yield, ?Await, ?Return] Catch[?Yield, ?Await, ?Return] Finally[?Yield, ?Await, ?Return]

    ...

    ExpressionStatement[Yield, Await]:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression[+In, ?Yield, ?Await];

---*/


var binding;

// [+Await]for await ( [lookahead ≠ let] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for await (binding of [await await await await await await await await await await await await await await await 'await']) {
  await await await await await await await await await await await await await await await 'await';
  break;
}

// [+Await]for await ( var ForBinding[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for await (var binding of [await await await await await await await await await await await await await await await 'await']) {
  await await await await await await await await await await await await await await await 'await';
  break;
}

// [+Await]for await ( ForDeclaration[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for await (let binding of [await await await await await await await await await await await await await await await 'await']) {
  await await await await await await await await await await await await await await await 'await';
  break;
}
