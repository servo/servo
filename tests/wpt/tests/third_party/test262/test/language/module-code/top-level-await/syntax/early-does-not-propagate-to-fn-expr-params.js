// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The Await capability does not propagate to the parameters of a function expression
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

  StatementListItem[Yield, Await, Return]:
    Statement[?Yield, ?Await, ?Return]
    Declaration[?Yield, ?Await]

  FunctionExpression:
    function BindingIdentifier[~Yield, ~Await]_opt ( FormalParameters[~Yield, ~Await] )
      { FunctionBody[~Yield, ~Await] }
negative:
  phase: parse
  type: SyntaxError
esid: prod-ModuleItem
flags: [module]
features: [top-level-await]
---*/

$DONOTEVALUATE();

0, function (x = await 1) {
  return x;
};
