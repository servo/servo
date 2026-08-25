// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Valid syntax for top level await.
  AwaitExpression ImportCall
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

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

  Catch[Yield, Await, Return]:
    catch(CatchParameter[?Yield, ?Await])Block[?Yield, ?Await, ?Return]
    catchBlock[?Yield, ?Await, ?Return]

  Finally[Yield, Await, Return]:
    finallyBlock[?Yield, ?Await, ?Return]

  CatchParameter[Yield, Await]:
    BindingIdentifier[?Yield, ?Await]
    BindingPattern[?Yield, ?Await]
esid: prod-AwaitExpression
flags: [module]
features: [top-level-await, dynamic-import]
---*/

try {} catch ({ x = await 42 }) {} // Initializer
try {} catch ({ x: y = await 42 }) {} // BindingElement Initializer
try {} catch ([ x = await 42 ]) {}
