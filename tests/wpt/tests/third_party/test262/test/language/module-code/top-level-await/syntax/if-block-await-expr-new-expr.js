// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-new-expr.case
// - src/top-level-await/syntax/if-block.template
/*---
description: AwaitExpression new MemberExpression (Valid syntax for top level await in an if expression position.)
esid: prod-AwaitExpression
features: [top-level-await]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    IfStatement[Yield, Await, Return]:
      if(Expression[+In, ?Yield, ?Await])Statement[?Yield, ?Await, ?Return]elseStatement[?Yield, ?Await, ?Return]
      if(Expression[+In, ?Yield, ?Await])Statement[?Yield, ?Await, ?Return]

    ...

    UnaryExpression[Yield, Await]
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


if (true) {
  await new Promise(function(res, rej) { res(1); });
}
