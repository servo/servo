// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped function arguments are wrapped into the inner realm
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();
const blueFn = (x, y) => x + y;

const redWrappedFn = r.evaluate(`
0, (blueWrappedFn, a, b, c) => {
    return blueWrappedFn(a, b) * c;
}
`);
assert.sameValue(redWrappedFn(blueFn, 2, 3, 4), 20);
