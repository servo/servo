// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Evaluation of await before a NewExpression
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

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
esid: prod-AwaitExpression
flags: [module, async]
features: [top-level-await]
---*/

var value = await new Promise(function(res, rej) {
  res(42);
});

assert.sameValue(value, 42);

$DONE();
