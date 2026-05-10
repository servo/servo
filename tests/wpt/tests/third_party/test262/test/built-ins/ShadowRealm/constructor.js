// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm-constructor
description: >
  ShadowRealm is a constructor and has [[Construct]] internal method.
includes: [isConstructor.js]
features: [ShadowRealm, Reflect.construct]
---*/
assert.sameValue(
  typeof ShadowRealm,
  'function',
  'This test must fail if ShadowRealm is not a function'
);

assert(isConstructor(ShadowRealm));
assert.sameValue(Object.getPrototypeOf(ShadowRealm), Function.prototype);
new ShadowRealm();
