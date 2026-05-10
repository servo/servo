// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped functions can resolve callable returns.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const wrapped = r.evaluate('x => y => x * y');
const nestedWrapped = wrapped(2);
const otherNestedWrapped = wrapped(4);

assert.sameValue(otherNestedWrapped(3), 12);
assert.sameValue(nestedWrapped(3), 6);

assert.notSameValue(nestedWrapped, otherNestedWrapped, 'new wrapping for each return');
