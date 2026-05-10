// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Verify a RegularExpressionLiteral following an AwaitExpression is
  not ambiguous to an Division
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

  ...

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

var lol = false;
var x = {
  get y() {
    lol = true;
  }
};

var g = 42;

await /x.y/g;

if (lol) {
  $DONE('It should be a RegExp');
} else {
  $DONE();
}
