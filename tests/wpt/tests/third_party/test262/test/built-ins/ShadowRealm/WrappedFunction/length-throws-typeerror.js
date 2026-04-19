// Copyright (C) 2022 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrappedfunctioncreate
description: >
  WrappedFunctionCreate throws a TypeError from its caller realm.
info: |
  WrappedFunctionCreate ( callerRealm: a Realm Record, Target: a function object, )

  ...
  7. Let result be CopyNameAndLength(wrapped, Target).
  8. If result is an Abrupt Completion, throw a TypeError exception.
  ...

features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.throws(TypeError, () => r.evaluate(`
function fn() {}
Object.defineProperty(fn, 'length', {
  get: () => {
    throw new Error('blah');
  },
  enumerable: false,
  configurable: true,
});
fn;
`), 'expect a TypeError on length getter throwing');
