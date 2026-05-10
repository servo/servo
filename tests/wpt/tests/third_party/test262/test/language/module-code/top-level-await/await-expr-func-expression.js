// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  A function after top level await is an expression and not a hoistable declaration
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

  ...

  ExpressionStatement[Yield, Await]:
    [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
      Expression[+In, ?Yield, ?Await];

  UnaryExpression[Yield, Await]
    [+Await]AwaitExpression[?Yield]

  AwaitExpression[Yield]:
    await UnaryExpression[?Yield, +Await]

  ...

  PrimaryExpression[Yield, Await]:
    this
    IdentifierReference[?Yield, ?Await]
    Literal
    ArrayLiteral[?Yield, ?Await]
    ObjectLiteral[?Yield, ?Await]
    FunctionExpression
    ClassExpression[?Yield, ?Await]
    GeneratorExpression
    AsyncFunctionExpression
    AsyncGeneratorExpression
    RegularExpressionLiteral
    TemplateLiteral[?Yield, ?Await, ~Tagged]
    CoverParenthesizedExpressionAndArrowParameterList[?Yield, ?Await]
esid: prod-AwaitExpression
flags: [module, async]
features: [top-level-await]
---*/

function fn() { return 42; }
await function fn() { return 111; };

assert.sameValue(fn(), 42);

$DONE();
