// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-nested.case
// - src/top-level-await/syntax/try.template
/*---
description: Nested AwaitExpressions (Valid syntax for top level await in try-catch-finally blocks.)
esid: prod-AwaitExpression
features: [top-level-await]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    TryStatement[Yield, Await, Return]:
      try Block[?Yield, ?Await, ?Return] Catch[?Yield, ?Await, ?Return]
      try Block[?Yield, ?Await, ?Return] Finally[?Yield, ?Await, ?Return]
      try Block[?Yield, ?Await, ?Return] Catch[?Yield, ?Await, ?Return] Finally[?Yield, ?Await, ?Return]

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


try {
  await await await await await await await await await await await await await await await 'await';
} catch(e) {
  await await await await await await await await await await await await await await await 'await';
}

try {
  await await await await await await await await await await await await await await await 'await';
} finally {
  await await await await await await await await await await await await await await await 'await';
}

try {
  await await await await await await await await await await await await await await await 'await';
} catch(e) {
  await await await await await await await await await await await await await await await 'await';
} finally {
  await await await await await await await await await await await await await await await 'await';
}
