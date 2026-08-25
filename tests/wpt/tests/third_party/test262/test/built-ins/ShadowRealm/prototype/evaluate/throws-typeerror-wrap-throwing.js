// Copyright (C) 2022 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrappedfunctioncreate
description: >
    WrappedFunctionCreate throws a TypeError if the accessing target's property may throw.

info: |
  WrappedFunctionCreate ( callerRealm: a Realm Record, Target: a function object, )
  ...
  7. Let result be CopyNameAndLength(wrapped, Target).
  ...

  CopyNameAndLength ( F: a function object, Target: a function object, optional prefix: a String, optional argCount: a Number, )
  ...
  3. Let targetHasLength be ? HasOwnProperty(Target, "length").
  4. If targetHasLength is true, then
    a. Let targetLen be ? Get(Target, "length").
  ...
  6. Let targetName be ? Get(Target, "name").

features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.throws(TypeError, () => r.evaluate(`
const revocable = Proxy.revocable(() => {}, {});
revocable.revoke();

revocable.proxy;
`), 'TypeError on wrapping a revoked callable proxy');

assert.throws(TypeError, () => r.evaluate(`
const fn = () => {};
Object.defineProperty(fn, 'name', {
  get() {
    throw new Error();
  },
});

fn;
`), 'TypeError on wrapping a fn with throwing name accessor');

assert.throws(TypeError, () => r.evaluate(`
const fn = () => {};
Object.defineProperty(fn, 'length', {
  get() {
    throw new Error();
  },
});

fn;
`), 'TypeError on wrapping a fn with throwing length accessor');

assert.throws(TypeError, () => r.evaluate(`
const proxy = new Proxy(() => {}, {
  getOwnPropertyDescriptor(target, key) {
    throw new Error();
  },
});

proxy;
`), 'TypeError on wrapping a callable proxy with throwing getOwnPropertyDescriptor trap');
