// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  AwaitExpression Resolutions
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

var x;

x = await 42;
assert.sameValue(x, 42, 'number');

x = await '';
assert.sameValue(x, '', 'string');

var s = Symbol();
x = await s;
assert.sameValue(x, s, 'symbol');

x = await false;
assert.sameValue(x, false, 'false');

x = await true;
assert.sameValue(x, true, 'true');

x = await NaN;
assert.sameValue(x, NaN, 'NaN');

x = await null;
assert.sameValue(x, null, 'null');

x = await undefined;
assert.sameValue(x, undefined, 'undefined');

var obj = {};
x = await obj;
assert.sameValue(x, obj, 'object');

x = await Promise.resolve(1).then(v => v * 2).then(v => v * 3);
assert.sameValue(x, 6, 'promise');

$DONE();
