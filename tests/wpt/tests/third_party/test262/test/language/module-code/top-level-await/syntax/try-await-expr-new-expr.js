// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-new-expr.case
// - src/top-level-await/syntax/try.template
/*---
description: AwaitExpression new MemberExpression (Valid syntax for top level await in try-catch-finally blocks.)
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


try {
  await new Promise(function(res, rej) { res(1); });
} catch(e) {
  await new Promise(function(res, rej) { res(1); });
}

try {
  await new Promise(function(res, rej) { res(1); });
} finally {
  await new Promise(function(res, rej) { res(1); });
}

try {
  await new Promise(function(res, rej) { res(1); });
} catch(e) {
  await new Promise(function(res, rej) { res(1); });
} finally {
  await new Promise(function(res, rej) { res(1); });
}
