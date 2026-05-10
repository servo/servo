// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  void AwaitExpression is still evaluated
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

  ...

  UnaryExpression[Yield, Await]
    void UnaryExpression[?Yield, ?Await]
    [+Await]AwaitExpression[?Yield]

  AwaitExpression[Yield]:
    await UnaryExpression[?Yield, +Await]
esid: prod-AwaitExpression
flags: [module, async]
features: [top-level-await]
---*/

var got = 0;
var x = {
  get y() {
    got += 1;
  }
};

void await x.y;

assert.sameValue(got, 1);

$DONE();
