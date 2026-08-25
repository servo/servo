// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  AwaitExpression evaluates to abrupt completions in promise rejections
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

try {
  await Promise.reject(42);
} catch (e) {
  x = e;
}
assert.sameValue(x, 42, 'number');

try {
  await Promise.reject('');
} catch (e) {
  x = e;
}
assert.sameValue(x, '', 'string');

try {
  var s = Symbol();
  await Promise.reject(s);
} catch (e) {
  x = e;
}
assert.sameValue(x, s, 'symbol');

try {
  await Promise.reject(false);
} catch (e) {
  x = e;
}
assert.sameValue(x, false, 'false');

try {
  await Promise.reject(true);
} catch (e) {
  x = e;
}
assert.sameValue(x, true, 'true');

try {
  await Promise.reject(NaN);
} catch (e) {
  x = e;
}
assert.sameValue(x, NaN, 'NaN');

try {
  await Promise.reject(null);
} catch (e) {
  x = e;
}
assert.sameValue(x, null, 'null');

try {
  await Promise.reject(undefined);
} catch (e) {
  x = e;
}
assert.sameValue(x, undefined, 'undefined');

try {
  var obj = {};
  await Promise.reject(obj);
} catch (e) {
  x = e;
}
assert.sameValue(x, obj, 'object');

$DONE();
