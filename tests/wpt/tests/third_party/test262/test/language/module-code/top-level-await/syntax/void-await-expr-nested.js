// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-nested.case
// - src/top-level-await/syntax/void.template
/*---
description: Nested AwaitExpressions (Valid syntax for top level await in an UnaryExpression (void).)
esid: prod-AwaitExpression
features: [top-level-await]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    UnaryExpression[Yield, Await]
      void UnaryExpression[?Yield, ?Await]
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


void await await await await await await await await await await await await await await await 'await';
