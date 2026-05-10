// Copyright (C) 2022 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrapped-function-exotic-objects-call-thisargument-argumentslist
description: >
    WrappedFunctionCreate throws a TypeError the target is a revoked proxy.

info: |
  WrappedFunctionCreate ( callerRealm: a Realm Record, Target: a function object, )
  1. Let target be F.[[WrappedTargetFunction]].
  2. Assert: IsCallable(target) is true.
  3. Let callerRealm be F.[[Realm]].
  4. NOTE: Any exception objects produced after this point are associated with callerRealm.
  5. Let targetRealm be ? GetFunctionRealm(target).
  ...

  GetFunctionRealm ( obj )
  ...
  3. If obj is a Proxy exotic object, then
    a. If obj.[[ProxyHandler]] is null, throw a TypeError exception.
  ...

features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const fn = r.evaluate(`
globalThis.revocable = Proxy.revocable(() => {}, {});

globalThis.revocable.proxy;
`);
r.evaluate('revocable.revoke()');
assert.throws(TypeError, () => fn());
