// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrapped-function-exotic-objects-call-thisargument-argumentslist
description: >
  WrappedFunction throws a TypeError if any of the arguments are non-primitive
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();
const wrappedFunction = r.evaluate('() => {}');

assert.sameValue(
  typeof wrappedFunction,
  'function',
  'This test must fail if wrappedFunction is not a function'
);

assert.throws(TypeError, () => wrappedFunction(1, globalThis), 'globalThis');
assert.throws(TypeError, () => wrappedFunction(1, []), 'array literal');
assert.throws(TypeError, () => wrappedFunction(1, {
  [Symbol.toPrimitive]() { return 'string'; },
  toString() { return 'str'; },
  valueOf() { return 1; }
}), 'object literal with immediate primitive coercion methods');
assert.throws(TypeError, () => wrappedFunction(1, Object.create(null)), 'ordinary object with null __proto__');
assert.throws(TypeError, () => wrappedFunction(1, new Proxy({}, { apply() {} })), 'non-callable proxy');
