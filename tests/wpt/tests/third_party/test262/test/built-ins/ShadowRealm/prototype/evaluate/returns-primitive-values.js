// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate returns primitive values
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.sameValue(r.evaluate('1 + 1'), 2);
assert.sameValue(r.evaluate('null'), null);
assert.sameValue(r.evaluate(''), undefined, 'undefined from empty completion');
assert.sameValue(r.evaluate('undefined'), undefined);
assert.sameValue(r.evaluate('true'), true);
assert.sameValue(r.evaluate('false'), false);
assert.sameValue(r.evaluate('function fn() {}'), undefined, 'fn declaration has empty completion');
assert.sameValue(r.evaluate('{}'), undefined, 'Block has empty completion');
assert.sameValue(r.evaluate('-0'), -0);
assert.sameValue(r.evaluate('"str"'), 'str');
assert(Number.isNaN(r.evaluate('NaN')));
