// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate is not a constructor.
includes: [isConstructor.js]
features: [ShadowRealm, Reflect.construct]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

assert.sameValue(
  isConstructor(ShadowRealm.prototype.evaluate),
  false,
  'isConstructor(ShadowRealm.prototype.evaluate) must return false'
);

assert.throws(TypeError, () => {
  new ShadowRealm.prototype.evaluate("");
});

const r = new ShadowRealm();
r.evaluate('globalThis.x = 0');

assert.throws(TypeError, () => {
  new r.evaluate("globalThis.x += 1;");
}, '`new r.evaluate("...")` throws TypeError');

assert.sameValue(r.evaluate('globalThis.x'), 0, 'No code evaluated in the new expression');
