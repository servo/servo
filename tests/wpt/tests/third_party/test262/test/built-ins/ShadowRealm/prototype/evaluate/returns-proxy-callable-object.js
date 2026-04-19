// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped proxy callable object.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const proxyCallable = r.evaluate(`
function fn() { return 42; }
new Proxy(fn, {});
`);

assert.sameValue(typeof proxyCallable, 'function', 'wrapped proxy callable object is typeof function');
assert.sameValue(proxyCallable(), 42, 'wrappedpfn() returns 42');
assert.sameValue((new Proxy(proxyCallable, {}))(), 42, 'wrapped functions can be proxied');
