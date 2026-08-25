// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-new-expr.case
// - src/top-level-await/syntax/typeof.template
/*---
description: AwaitExpression new MemberExpression (Valid syntax for top level await in an UnaryExpression (void).)
esid: prod-AwaitExpression
features: [top-level-await]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    UnaryExpression[Yield, Await]
      typeof UnaryExpression[?Yield, ?Await]
      [+Await]AwaitExpression[?Yield]

    AwaitExpression[Yield]:
      await UnaryExpression[?Yield, +Await]

    ...


    LeftHandSideExpression[Yield, Await]:
      NewExpression[?Yield, ?Await]
      CallExpression[?Yield, ?Await]

    NewExpression[Yield, Await]:
      MemberExpression[?Yield, ?Await]
      new NewExpression[?Yield, ?Await]

    MemberExpression[Yield, Await]:
      ...
      new MemberExpression[?Yield, ?Await] Arguments[?Yield, ?Await]

---*/


typeof await new Promise(function(res, rej) { res(1); });
