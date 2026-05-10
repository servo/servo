// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm-constructor
description: >
  new ShadowRealm() returns a shadow realm instance
info: |
  ShadowRealm ( )

  ...
  2. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%ShadowRealm.prototype%",
  « [[ShadowRealm]], [[ExecutionContext]] »).
  ...
  13. Return O.
features: [ShadowRealm]
---*/
assert.sameValue(
  typeof ShadowRealm,
  'function',
  'This test must fail if ShadowRealm is not a function'
);

var realm = new ShadowRealm();

assert(realm instanceof ShadowRealm);
assert.sameValue(
  Object.getPrototypeOf(realm),
  ShadowRealm.prototype,
  '[[Prototype]] is set to %ShadowRealm.prototype%'
);

var otherRealm = new ShadowRealm();
assert.notSameValue(realm, otherRealm, 'each instance is different');
