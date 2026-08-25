// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate throws when argument is not a string.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.throws(TypeError, () => r.evaluate(['1+1']));
assert.throws(TypeError, () => r.evaluate({ [Symbol.toPrimitive]() { return '1+1'; }}));
assert.throws(TypeError, () => r.evaluate(1));
assert.throws(TypeError, () => r.evaluate(null));
assert.throws(TypeError, () => r.evaluate(undefined));
assert.throws(TypeError, () => r.evaluate(true));
assert.throws(TypeError, () => r.evaluate(false));
assert.throws(TypeError, () => r.evaluate(new String('nope')));
