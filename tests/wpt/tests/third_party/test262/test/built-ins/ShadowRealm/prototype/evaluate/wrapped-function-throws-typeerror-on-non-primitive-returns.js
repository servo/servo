// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrapped-function-exotic-objects-call-thisargument-argumentslist
description: >
  WrappedFunction throws a TypeError if it returns non-primitive values
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.throws(TypeError, r.evaluate('() => globalThis'), 'globalThis');
assert.throws(TypeError, r.evaluate('() => []'), 'array literal');
assert.throws(TypeError, r.evaluate(`
    () => ({
        [Symbol.toPrimitive]() { return 'string'; },
        toString() { return 'str'; },
        valueOf() { return 1; }
    });
`), 'object literal with immediate primitive coercion methods');
assert.throws(TypeError, r.evaluate('() => Object.create(null)'), 'ordinary object with null __proto__');
assert.throws(TypeError, r.evaluate('() => new Proxy({}, { apply() {} })'), 'non-callable proxy');
